//! `EntryArgs` field definitions for `uuid::Uuid`.
use darling::FromMeta;
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

    fn redb_ty(entry: &EntryArgs) -> RedbType {
        let ty = Type::from_string("&'static [u8; 16]").expect("Misconfigured type definition");

        RedbType::new(ty)
    }

    fn from_op(entry: &EntryArgs) -> Expr {
        Expr::from_string(format!("uuid::Uuid::from_bytes(*{})", entry.ident()).as_str())
            .expect("Misconfigured from operation")
    }

    fn into_op(entry: &EntryArgs) -> Expr {
        Expr::from_string(format!("{}.as_bytes()", entry.ident()).as_str())
            .expect("Misconfigured from operation")
    }
}
