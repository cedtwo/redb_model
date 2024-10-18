//! Internal type definitions.
use std::ops::Deref;

use darling::FromMeta;
use quote::format_ident;
use syn::{spanned::Spanned, Lifetime, Type};

/// The `redb` entry `Type` definition. Declares any lifetime as `'static`.
#[derive(Clone)]
pub(super) struct RedbType(Type);

impl Deref for RedbType {
    type Target = Type;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromMeta for RedbType {
    fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
        let mut ty = Type::from_meta(item)?;

        // Declare redb lifetimes as 'static`.
        if let Type::Reference(ref mut ref_ty) = ty {
            ref_ty.lifetime = Some(Lifetime {
                apostrophe: ref_ty.span(),
                ident: format_ident!("static"),
            })
        }

        Ok(RedbType(ty))
    }
}
