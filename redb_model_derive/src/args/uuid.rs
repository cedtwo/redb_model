//! `EntryArgs` field definitions for `uuid::Uuid`.
use proc_macro2::Literal;
use punctuated::Punctuated;
use quote::format_ident;
use syn::*;
use token::{Dot, Paren};

use super::external::ExternalType;
use crate::args::ty::RedbType;
use crate::args::EntryArgs;

/// `uuid::Uuid` marker type.
pub(super) struct UuidType;

impl ExternalType for UuidType {
    const IDENT_MATCH: &'static str = "Uuid";

    fn redb_ty(&self, entry: &EntryArgs) -> RedbType {
        // `&'static [u8; 16]`

        let mut segments = Punctuated::new();
        segments.push(PathSegment {
            ident: format_ident!("u8"),
            arguments: syn::PathArguments::None,
        });

        let ty = Type::Reference(TypeReference {
            and_token: Token![&](entry.ident().span()),
            lifetime: Some(Lifetime {
                apostrophe: entry.ident().span(),
                ident: format_ident!("static"),
            }),
            mutability: None,
            elem: Box::new(Type::Array(TypeArray {
                bracket_token: token::Bracket(entry.ident().span()),
                elem: Box::new(Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments,
                    },
                })),
                semi_token: Token![;](entry.ident().span()),
                len: Expr::Lit(ExprLit {
                    attrs: Vec::new(),
                    lit: Lit::new(Literal::usize_suffixed(16)),
                }),
            })),
        });

        RedbType::new(ty)
    }

    fn from_op(&self, entry: &EntryArgs) -> Expr {
        // `uuid::Uuid::from_bytes(*field)`

        let mut path = Path {
            leading_colon: None,
            segments: Punctuated::new(),
        };
        path.segments.push(PathSegment {
            ident: format_ident!("uuid"),
            arguments: syn::PathArguments::None,
        });
        path.segments.push(PathSegment {
            ident: format_ident!("Uuid"),
            arguments: syn::PathArguments::None,
        });
        path.segments.push(PathSegment {
            ident: format_ident!("from_bytes"),
            arguments: syn::PathArguments::None,
        });

        let mut args = Punctuated::new();
        let mut arg = Punctuated::new();
        arg.push(PathSegment {
            ident: entry.ident().clone(),
            arguments: syn::PathArguments::None,
        });
        args.push(Expr::Unary(ExprUnary {
            attrs: Vec::new(),
            op: syn::UnOp::Deref(Token![*](entry.ident().span())),
            expr: Box::new(Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: arg,
                },
            })),
        }));

        Expr::Call(ExprCall {
            attrs: Vec::new(),
            func: Box::new(Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path,
            })),
            paren_token: Paren(entry.ident().span()),
            args,
        })
    }

    fn into_op(&self, entry: &EntryArgs) -> Expr {
        // `field.as_bytes()`

        Expr::MethodCall(ExprMethodCall {
            attrs: vec![],
            receiver: Box::new(entry.ident_expr()),
            dot_token: Dot(entry.ident().span()),
            method: format_ident!("as_bytes"),
            turbofish: None,
            paren_token: Default::default(),
            args: Punctuated::new(),
        })
    }
}
