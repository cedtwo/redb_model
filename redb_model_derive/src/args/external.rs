//! Trait for defining `EntryArgs` for external types.
use syn::*;

use super::{ty::RedbType, EntryArgs};

/// Identifier, database type and field expressions for an external type.
pub(crate) trait ExternalType {
    /// An identifying literal to be found in the `Ident` path.
    const IDENT_MATCH: &'static str;

    /// Assert the given `Type` matches the external type.
    fn is_external_type(ty: &Type) -> bool {
        if let Type::Path(TypePath { path, .. }) = &ty {
            path.segments
                .iter()
                .any(|segment| &segment.ident.to_string() == Self::IDENT_MATCH)
        } else {
            false
        }
    }

    /// The variable type to declare within the database.
    fn redb_ty(&self, entry: &EntryArgs) -> RedbType;

    /// Type conversion operation **from** the `redb` type.
    fn from_op(&self, entry: &EntryArgs) -> Expr;

    /// Type conversion operation **into** the `redb` type.
    fn into_op(&self, entry: &EntryArgs) -> Expr;
}
