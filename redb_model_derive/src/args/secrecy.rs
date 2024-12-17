//! `EntryArgs` field definitions for `secrecy`.
use darling::FromMeta;
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
        RedbType::new(Type::from_string("&'static str").expect("Misconfigured from operation"))
    }

    fn from_op(entry: &EntryArgs) -> Expr {
        Expr::from_string(format!("secrecy::SecretString::from({})", entry.ident()).as_str())
            .expect("Misconfigured from operation")
    }

    fn into_op(entry: &EntryArgs) -> Expr {
        Expr::from_string(
            format!("secrecy::ExposeSecret::expose_secret({})", entry.ident()).as_str(),
        )
        .expect("Misconfigured from operation")
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
        Expr::from_string(format!("secrecy::SecretBox::new(Box::new({}))", entry.ident()).as_str())
            .expect("Misconfigured from operation")
    }

    fn into_op(entry: &EntryArgs) -> Expr {
        Expr::from_string(
            format!("*secrecy::ExposeSecret::expose_secret({})", entry.ident()).as_str(),
        )
        .expect("Misconfigured from operation")
    }
}
