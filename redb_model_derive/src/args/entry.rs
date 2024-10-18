use darling::{FromField, FromMeta};
use quote::format_ident;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Dot, Star};
use syn::{Expr, ExprMethodCall, ExprPath, ExprReference, ExprUnary, Path};
use syn::{Ident, PathSegment, Token, Type};

use super::ty::RedbType;

/// Arguments declared on a named struct field.
#[derive(FromField, Clone)]
#[darling(attributes(entry))]
pub struct EntryArgs {
    ident: Option<Ident>,
    ty: Type,

    /// The variable declared as either a `key` or `value`.
    position: EntryPosition,
    /// The type declared in the redb table definition.
    redb_type: Option<RedbType>,
    /// The method to call to resolve from the redb type.
    from: Option<Expr>,
    /// The method to call to resolve into the redb type.
    into: Option<Expr>,
}

#[derive(FromMeta, Clone, PartialEq, Eq)]
#[darling(rename_all = "lowercase")]
pub enum EntryPosition {
    Key,
    Value,
}

impl EntryArgs {
    /// Get the `VariablePosition`.
    pub fn position(&self) -> &EntryPosition {
        &self.position
    }

    /// The field name within the model.
    pub fn ident(&self) -> &Ident {
        &self.ident.as_ref().expect("Named struct")
    }

    /// The field name within the model as a reference.
    pub fn ident_ref(&self) -> Expr {
        Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Token![&](self.ident.span()),
            mutability: None,
            expr: Box::new(self.ident_expr()),
        })
    }

    /// The field name as an `Expr`.
    pub fn ident_expr(&self) -> Expr {
        let mut segments = Punctuated::new();
        segments.push_value(PathSegment {
            ident: self.ident().to_owned(),
            arguments: syn::PathArguments::None,
        });
        Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path: Path {
                leading_colon: None,
                segments,
            },
        })
    }

    /// The variable type declared within the model.
    pub fn model_ty(&self) -> &Type {
        &self.ty
    }

    /// The variable type to declare within the database.
    pub fn redb_ty(&self) -> &Type {
        match self.redb_type {
            Some(ref ty) => ty,
            None => &self.ty,
        }
    }

    /// Type conversion operation **from** the `redb` type.
    pub fn from_op(&self) -> Expr {
        self.from.clone().unwrap_or_else(|| {
            match &self.redb_type {
                Some(redb_type) if **redb_type != self.ty => {
                    match **redb_type {
                        // Call `to_owned` on the value.
                        Type::Reference(_) => Expr::MethodCall(ExprMethodCall {
                            attrs: vec![],
                            receiver: Box::new(self.ident_expr()),
                            dot_token: Dot(self.ident.span()),
                            method: format_ident!("to_owned"),
                            turbofish: None,
                            paren_token: Default::default(),
                            args: Punctuated::new(),
                        }),
                        // Call `into` on the value.
                        _ => Expr::MethodCall(ExprMethodCall {
                            attrs: vec![],
                            receiver: Box::new(self.ident_expr()),
                            dot_token: Dot(self.ident.span()),
                            method: format_ident!("into"),
                            turbofish: None,
                            paren_token: Default::default(),
                            args: Punctuated::new(),
                        }),
                    }
                }
                // Assume a `Copy` type and pass the value.
                _ => self.ident_expr(),
            }
        })
    }

    /// Type conversion operation **into** the `redb` type.
    pub fn into_op(&self) -> Expr {
        self.into.clone().unwrap_or_else(|| {
            match &self.redb_type {
                Some(redb_type) if **redb_type != self.ty => {
                    match **redb_type {
                        // Pass a reference to the value.
                        Type::Reference(_) => self.ident_expr(),
                        // Expr::Reference(ExprReference {
                        //     attrs: vec![],
                        //     and_token: Token![&](self.ident.span()),
                        //     mutability: None,
                        //     expr: Box::new(self.ident_expr()),
                        // }),

                        // Call `into` on the value.
                        _ => Expr::MethodCall(ExprMethodCall {
                            attrs: vec![],
                            receiver: Box::new(self.ident_expr()),
                            dot_token: Dot(self.ident.span()),
                            method: format_ident!("into"),
                            turbofish: None,
                            paren_token: Default::default(),
                            args: Punctuated::new(),
                        }),
                    }
                }
                // Assume a `Copy` type and dereference the value.
                _ => Expr::Unary(ExprUnary {
                    attrs: vec![],
                    op: syn::UnOp::Deref(Star(self.ident().span())),
                    expr: Box::new(self.ident_expr()),
                }),
            }
        })
    }
}
