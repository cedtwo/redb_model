//! Table variable interpolation.
use darling::util::Shape;
use quote::quote;
use syn::{Ident, Lifetime, Type, TypeReference, TypeTuple};

/// A variable, or tuple of variables.
pub(super) struct CompositeVariable {
    /// The type or a tuple of types.
    ty: Type,
    /// The type reference or a tuple of type references.
    ty_ref: Type,
    /// All idents of the variable.
    idents: Vec<Ident>,
}

impl CompositeVariable {
    /// Create a new `CompositeVariable`, from an iterator `FieldArgs`. A single
    /// variable will remain untouched, while multiple variables will be resolved
    /// as tuples.
    pub(crate) fn new<I>(fields: I) -> Result<Self, darling::Error>
    where
        I: IntoIterator<Item = super::args::VariableArgs, IntoIter: ExactSizeIterator>,
    {
        let mut iter = fields
            .into_iter()
            .try_fold(Vec::new(), |mut v, args| match args.ident {
                Some(ident) => {
                    v.push((args.ty, ident));
                    Ok(v)
                }
                None => Err(darling::Error::unsupported_shape_with_expected(
                    "Unnamed struct",
                    &Shape::Named,
                )),
            })?;

        let (ty, ty_ref, idents) = match iter.len() {
            1 => {
                let (ty, ident) = iter.swap_remove(0);
                let ty_ref = Type::Reference(TypeReference {
                    and_token: syn::Token![&](ident.span()),
                    lifetime: Some(Lifetime::new("'a", ident.span())),
                    mutability: None,
                    elem: Box::new(ty.clone()),
                });

                (ty, ty_ref, vec![ident])
            }
            _ => {
                let mut ty_el = syn::punctuated::Punctuated::new();
                let mut ty_ref_el = syn::punctuated::Punctuated::new();
                let mut idents = Vec::with_capacity(iter.len());

                iter.into_iter().for_each(|args| {
                    ty_el.push(args.0.clone());
                    ty_ref_el.push(Type::Reference(TypeReference {
                        and_token: syn::Token![&](args.1.span()),
                        lifetime: Some(Lifetime::new("'a", args.1.span())),
                        mutability: None,
                        elem: Box::new(args.0),
                    }));
                    idents.push(args.1);
                });

                let ty = Type::Tuple(TypeTuple {
                    paren_token: Default::default(),
                    elems: ty_el,
                });
                let ty_ref = Type::Tuple(TypeTuple {
                    paren_token: Default::default(),
                    elems: ty_ref_el,
                });

                (ty, ty_ref, idents)
            }
        };

        Ok(Self { ty, ty_ref, idents })
    }

    pub(crate) fn ty(&self) -> &Type {
        &self.ty
    }

    pub(crate) fn ty_ref(&self) -> &Type {
        &self.ty_ref
    }

    pub(crate) fn idents(&self) -> &Vec<Ident> {
        &self.idents
    }

    pub(crate) fn composite_ident(
        &self,
        prefix: Option<proc_macro2::TokenStream>,
    ) -> proc_macro2::TokenStream {
        let prefix = prefix.unwrap_or(quote!());
        let idents = self.idents();

        if idents.len() == 1 {
            quote! { #( #prefix #idents ) * }
        } else {
            quote! { ( #( #prefix #idents ), * ) }
        }
    }
}
