//! `EntryArgs` field definitions for `secrecy`.
use punctuated::Punctuated;
use quote::format_ident;
use syn::*;
use token::Paren;

use super::external::ExternalType;
use crate::args::ty::RedbType;
use crate::args::EntryArgs;

/// `secrecy::SecretString` marker type.
pub(super) struct SecretStringType;

impl ExternalType for SecretStringType {
    const IDENT_MATCH: &'static str = "SecretString";

    fn redb_ty(entry: &EntryArgs) -> RedbType {
        // `&'static str`

        let mut path = Path {
            leading_colon: None,
            segments: Punctuated::new(),
        };
        path.segments.push(PathSegment {
            ident: format_ident!("str"),
            arguments: syn::PathArguments::None,
        });

        let ty = Type::Reference(TypeReference {
            and_token: Token![&](entry.ident().span()),
            lifetime: Some(Lifetime {
                apostrophe: entry.ident().span(),
                ident: format_ident!("static"),
            }),
            mutability: None,
            elem: Box::new(Type::Path(TypePath { qself: None, path })),
        });

        RedbType::new(ty)
    }

    fn from_op(entry: &EntryArgs) -> Expr {
        // `secrecy::SecretString::from(*field)`

        let mut path = Path {
            leading_colon: None,
            segments: Punctuated::new(),
        };
        path.segments.push(PathSegment {
            ident: format_ident!("secrecy"),
            arguments: syn::PathArguments::None,
        });
        path.segments.push(PathSegment {
            ident: format_ident!("SecretString"),
            arguments: syn::PathArguments::None,
        });
        path.segments.push(PathSegment {
            ident: format_ident!("from"),
            arguments: syn::PathArguments::None,
        });

        let mut args = Punctuated::new();
        let mut arg = Punctuated::new();
        arg.push(PathSegment {
            ident: entry.ident().clone(),
            arguments: syn::PathArguments::None,
        });
        args.push(Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: Path {
                leading_colon: None,
                segments: arg,
            },
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

    fn into_op(entry: &EntryArgs) -> Expr {
        // `secrecy::ExposeSecret::expose_secret(&field)`

        let mut path = Path {
            leading_colon: None,
            segments: Punctuated::new(),
        };
        path.segments.push(PathSegment {
            ident: format_ident!("secrecy"),
            arguments: syn::PathArguments::None,
        });
        path.segments.push(PathSegment {
            ident: format_ident!("ExposeSecret"),
            arguments: syn::PathArguments::None,
        });
        path.segments.push(PathSegment {
            ident: format_ident!("expose_secret"),
            arguments: syn::PathArguments::None,
        });

        let mut args = Punctuated::new();
        let mut arg = Punctuated::new();
        arg.push(PathSegment {
            ident: entry.ident().clone(),
            arguments: syn::PathArguments::None,
        });
        args.push(Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: Path {
                leading_colon: None,
                segments: arg,
            },
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
}

/// `secrecy::SecretBox` marker type.
pub(super) struct SecretBoxType;

impl ExternalType for SecretBoxType {
    const IDENT_MATCH: &'static str = "SecretBox";

    fn redb_ty(entry: &EntryArgs) -> RedbType {
        // `&'static S` for the given `SecretBox<S>`

        match entry.model_ty() {
            Type::Path(TypePath { path, .. }) => {
                if let Some(PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args,
                    ..
                })) = path
                    .segments
                    .iter()
                    .last()
                    .map(|segment| &segment.arguments)
                {
                    if let Some(GenericArgument::Type(ty)) = args.into_iter().next() {
                        return RedbType::new(ty.clone());
                    }
                }
            }
            _ => unreachable!("Unexpected type"),
        }

        panic!("No generic arguments found for `SecretBox` type")
    }

    fn from_op(entry: &EntryArgs) -> Expr {
        // `secrecy::SecretBox::new(Box::new(*field))`

        let mut box_path = Path {
            leading_colon: None,
            segments: Punctuated::new(),
        };
        box_path.segments.push(PathSegment {
            ident: format_ident!("Box"),
            arguments: syn::PathArguments::None,
        });
        box_path.segments.push(PathSegment {
            ident: format_ident!("new"),
            arguments: syn::PathArguments::None,
        });

        let mut box_args = Punctuated::new();
        let mut segments = Punctuated::new();
        segments.push(PathSegment {
            ident: entry.ident().clone(),
            arguments: syn::PathArguments::None,
        });
        box_args.push(Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: Path {
                leading_colon: None,
                segments,
            },
        }));

        let mut secrecy_path = Path {
            leading_colon: None,
            segments: Punctuated::new(),
        };
        secrecy_path.segments.push(PathSegment {
            ident: format_ident!("secrecy"),
            arguments: syn::PathArguments::None,
        });
        secrecy_path.segments.push(PathSegment {
            ident: format_ident!("SecretBox"),
            arguments: syn::PathArguments::None,
        });
        secrecy_path.segments.push(PathSegment {
            ident: format_ident!("new"),
            arguments: syn::PathArguments::None,
        });

        let mut args = Punctuated::new();
        args.push(Expr::Call(ExprCall {
            attrs: Vec::new(),
            func: Box::new(Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: box_path,
            })),
            paren_token: Paren(entry.ident().span()),
            args: box_args,
        }));

        Expr::Call(ExprCall {
            attrs: Vec::new(),
            func: Box::new(Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: secrecy_path,
            })),
            paren_token: Paren(entry.ident().span()),
            args,
        })
    }

    fn into_op(entry: &EntryArgs) -> Expr {
        // `*secrecy::ExposeSecret::expose_secret(&field)`

        let mut path = Path {
            leading_colon: None,
            segments: Punctuated::new(),
        };
        path.segments.push(PathSegment {
            ident: format_ident!("secrecy"),
            arguments: syn::PathArguments::None,
        });
        path.segments.push(PathSegment {
            ident: format_ident!("ExposeSecret"),
            arguments: syn::PathArguments::None,
        });
        path.segments.push(PathSegment {
            ident: format_ident!("expose_secret"),
            arguments: syn::PathArguments::None,
        });

        let mut args = Punctuated::new();
        let mut arg = Punctuated::new();
        arg.push(PathSegment {
            ident: entry.ident().clone(),
            arguments: syn::PathArguments::None,
        });
        args.push(Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: Path {
                leading_colon: None,
                segments: arg,
            },
        }));

        Expr::Unary(ExprUnary {
            attrs: Vec::new(),
            op: syn::UnOp::Deref(Token![*](entry.ident().span())),
            expr: Box::new(Expr::Call(ExprCall {
                attrs: Vec::new(),
                func: Box::new(Expr::Path(ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                })),
                paren_token: Paren(entry.ident().span()),
                args,
            })),
        })
    }
}
