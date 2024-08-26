//! Table metadata interpolation.
use syn::{
    punctuated::Punctuated,
    token::{Comma, PathSep},
    AngleBracketedGenericArguments, GenericArgument, Ident, Lifetime, Path, PathArguments,
    PathSegment, Token, Type, TypePath,
};

use crate::args::ModelTableType;

/// Metadata for table definitions.
pub(super) struct TableMetadata {
    ident: Ident,
    name: String,
    ty: ModelTableType,
}

impl TableMetadata {
    /// Create a new `TableMetaData` instance from the table `ident`, `name` and `ty`.
    pub(super) fn new(ident: Ident, name: String, ty: ModelTableType) -> Self {
        Self { ident, name, ty }
    }

    /// Get the table `Ident`.
    pub(super) fn ident(&self) -> &Ident {
        &self.ident
    }

    /// Get the table `name`.
    pub(super) fn name(&self) -> &str {
        &self.name
    }

    /// The table, or multimap table definition as a generic type.
    pub(crate) fn type_as_generic(&self, k: &Type, v: &Type) -> Type {
        // Generic argumemnts.
        let mut args: Punctuated<_, Comma> = Punctuated::new();
        args.push(GenericArgument::Lifetime(Lifetime::new(
            "'a",
            self.ident.span(),
        )));
        args.push(GenericArgument::Type(k.clone()));
        args.push(GenericArgument::Type(v.clone()));
        let table_generics = PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: Token![<](self.ident.span()),
            args,
            gt_token: Token![>](self.ident.span()),
        });

        // Path segments.
        let mut segments: Punctuated<PathSegment, PathSep> = Punctuated::new();
        segments.push(PathSegment {
            ident: Ident::new("redb", self.ident.span()),
            arguments: syn::PathArguments::None,
        });
        match self.ty {
            ModelTableType::Table => {
                segments.push(PathSegment {
                    ident: Ident::new("TableDefinition", self.ident.span()),
                    arguments: table_generics,
                });
            }
            ModelTableType::MultimapTable => {
                segments.push(PathSegment {
                    ident: Ident::new("MultimapTableDefinition", self.ident.span()),
                    arguments: table_generics,
                });
            }
        }

        Type::Path(TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments,
            },
        })
    }
}
