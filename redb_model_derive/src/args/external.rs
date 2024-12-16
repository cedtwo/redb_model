//! Trait for defining `EntryArgs` for external types.
use syn::*;

use super::{ty::RedbType, EntryArgs};

/// Identifier, database type and field expressions for an external type.
pub(crate) trait ExternalType {
    /// An identifying literal to be found in the `Ident` path.
    const IDENT_MATCH: &'static str;

    /// The variable type to declare within the database.
    fn redb_ty(entry: &EntryArgs) -> RedbType;

    /// Type conversion operation **from** the `redb` type.
    fn from_op(entry: &EntryArgs) -> Expr;

    /// Type conversion operation **into** the `redb` type.
    fn into_op(entry: &EntryArgs) -> Expr;
}
