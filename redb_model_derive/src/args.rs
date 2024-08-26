//! Derive macro input arguments.
use darling::ast::Data;
use darling::util::Ignored;
use darling::{FromDeriveInput, FromField, FromMeta};
use syn::{Ident, Type};

#[derive(FromDeriveInput)]
#[darling(attributes(model))]
pub struct ModelArgs {
    pub ident: Ident,
    pub data: Data<Ignored, VariableArgs>,

    /// The type of table to define.
    pub table_type: Option<ModelTableType>,
    /// The name of the table, defaulting to the struct `Ident`.
    pub name: Option<String>,
    /// Implement `ModelExt` for the given model.
    pub impl_ext: Option<bool>,
    /// Implement `From<T>` for the given model. Requires implementing `ModelExt`.
    pub impl_from: Option<bool>,
}

#[derive(FromMeta, Default)]
pub enum ModelTableType {
    #[default]
    Table,
    MultimapTable,
}

#[derive(FromField)]
#[darling(attributes(entry))]
pub struct VariableArgs {
    pub ident: Option<Ident>,
    pub ty: Type,

    /// The position of a variable in the entry.
    pub position: VariablePosition,
}

#[derive(FromMeta, PartialEq, Eq)]
#[darling(rename_all = "lowercase")]
pub enum VariablePosition {
    Key,
    Value,
}
