use darling::ast::Data;
use darling::util::Ignored;
use darling::{FromDeriveInput, FromMeta};
use syn::Ident;

use super::EntryArgs;

/// Arguments declared on a named struct.
#[derive(FromDeriveInput)]
#[darling(attributes(model))]
pub(crate) struct ModelArgs {
    pub ident: Ident,
    pub data: Data<Ignored, EntryArgs>,

    /// The table type, either `table` or `multimap`.
    pub table_type: Option<ModelTableType>,
    /// The name of the table, defaulting to the struct `Ident`.
    pub name: Option<String>,
    /// Implement `ModelExt` for the given model.
    pub impl_ext: Option<bool>,
    /// Implement `From<T>` for the given model. Requires implementing `ModelExt`.
    pub impl_from: Option<bool>,
}

#[derive(FromMeta, Default)]
pub(crate) enum ModelTableType {
    #[default]
    Table,
    Multimap,
}
