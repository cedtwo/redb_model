//! Table variable interpolation.
use std::ops::Deref;

use syn::{Expr, ExprTuple, Ident, Type, TypeTuple};

use crate::args::EntryArgs;

/// Metadata for table key/value composite type(s).
pub(super) struct ValueMeta<'a>(Vec<&'a EntryArgs>);

impl<'a> Deref for ValueMeta<'a> {
    type Target = Vec<&'a EntryArgs>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> ValueMeta<'a> {
    /// Create a new `ValueMeta`, from an iterator of `EntryArgs` references.
    pub(crate) fn new<I: 'a>(fields: I) -> Self
    where
        I: IntoIterator<Item = &'a EntryArgs>,
    {
        Self(fields.into_iter().collect())
    }

    /// Create a new `ValueMeta`, borrowing from two `ValueMeta` instances.
    pub(crate) fn new_merged(a: &'a Self, b: &'a Self) -> Self {
        Self::new(a.iter().chain(b.iter()).map(|var| *var))
    }

    /// The type, or a tuple of the types within the model.
    pub(crate) fn model_ty(&self) -> Type {
        match self.0.len() {
            1 => self[0].model_ty().clone(),
            _ => {
                let mut ty_el = syn::punctuated::Punctuated::new();
                ty_el.extend(self.iter().map(|var| var.model_ty().to_owned()));
                Type::Tuple(TypeTuple {
                    paren_token: Default::default(),
                    elems: ty_el,
                })
            }
        }
    }

    /// The type, or a tuple of the types within the database table definition.
    pub(crate) fn redb_ty(&self) -> Type {
        match self.len() {
            1 => self[0].redb_ty().clone(),
            _ => {
                let mut ty_el = syn::punctuated::Punctuated::new();
                ty_el.extend(self.iter().map(|var| var.redb_ty().to_owned()));
                Type::Tuple(TypeTuple {
                    paren_token: Default::default(),
                    elems: ty_el,
                })
            }
        }
    }

    /// Get an `Expr` of all idents.
    pub(crate) fn idents(&self) -> impl ExactSizeIterator<Item = &Ident> {
        self.iter().map(|var| var.ident())
    }

    /// Get an `Expr` of idents as either a single ident, or a tuple of idents.
    pub(crate) fn composite_idents(&self) -> Expr {
        match self.len() {
            1 => self[0].ident_expr(),
            _ => {
                let mut elems = syn::punctuated::Punctuated::new();
                elems.extend(self.iter().map(|var| var.ident_expr()));
                Expr::Tuple(ExprTuple {
                    attrs: vec![],
                    paren_token: Default::default(),
                    elems,
                })
            }
        }
    }

    /// Get an `Expr` of ident `from` calls.
    pub(crate) fn from_methods(&'a self) -> impl ExactSizeIterator<Item = Expr> + 'a {
        self.iter().map(|var| var.from_op())
    }

    /// Get an `Expr` of ident `borrow` calls.
    pub(crate) fn into_methods(&'a self) -> impl ExactSizeIterator<Item = Expr> + 'a {
        self.iter().map(|var| var.into_op())
    }
}
