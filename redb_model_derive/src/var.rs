//! Table variable interpolation.
use darling::util::Shape;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::PathSep;
use syn::{Ident, Lifetime, Path, PathSegment, Token, Type, TypePath, TypeReference, TypeTuple};

/// A variable, or tuple of variables.
///
/// This represents either the `K` or `V` generic for a table. For composite keys
/// or values, they are represented here as a tuple of all values. `String` types
/// are represented as string slices `&str` to simplify interop with `redb`.
pub(super) struct CompositeVariable {
    /// The type or a tuple of types.
    ty: Type,
    /// The type reference or a tuple of type references.
    ty_ref: Type,
    /// The `'static` type reference or a tuple of `'static` type references.
    ty_static: Type,
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
        let fields = fields
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

        let (ty, ty_ref, ty_static, idents) = Self::to_composite_types(fields);

        Ok(Self {
            ty,
            ty_ref,
            ty_static,
            idents,
        })
    }

    /// Get the type. or a tuple of the types.
    pub(crate) fn ty(&self) -> &Type {
        &self.ty
    }

    /// Get the type reference, or a tuple of type references.
    pub(crate) fn ty_ref(&self) -> &Type {
        &self.ty_ref
    }

    /// Get the type as a `'static` reference, or a tuple of the types as `'static` references.
    pub(crate) fn ty_static(&self) -> &Type {
        &self.ty_static
    }

    /// Get a vector of the idents.
    pub(crate) fn idents(&self) -> &Vec<Ident> {
        &self.idents
    }

    /// Get the `Ident`, or a tuple of the `Ident`'s, optionally prefixing each variable.
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

    /// Returns a single type, or
    fn to_composite_types(mut vec: Vec<(Type, Ident)>) -> (Type, Type, Type, Vec<Ident>) {
        let (ty, ty_ref, ty_static, idents) = match vec.len() {
            1 => {
                let (ty, ident) = vec.swap_remove(0);
                let (ty_ref, ty_static) = if ty.is_string() {
                    let mut segments = Punctuated::new();
                    segments.push(PathSegment {
                        ident: Ident::new("str", ty.span()),
                        arguments: syn::PathArguments::None,
                    });
                    let str_ty = Type::Path(TypePath {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: segments.clone(),
                        },
                    });
                    (
                        str_ty.clone().into_reference("'a"),
                        str_ty.into_reference("'static"),
                    )
                } else {
                    (
                        ty.clone().into_reference("'a"),
                        ty.clone().into_reference("'static"),
                    )
                };

                (ty, ty_ref, ty_static, vec![ident])
            }
            _ => {
                let mut ty_el = syn::punctuated::Punctuated::new();
                let mut ty_ref_el = syn::punctuated::Punctuated::new();
                let mut ty_static_el = syn::punctuated::Punctuated::new();
                let mut idents = Vec::with_capacity(vec.len());

                vec.into_iter().for_each(|args| {
                    let (ty, ident) = (args.0, args.1);
                    ty_el.push(ty.clone());

                    let (ty_ref, ty_static) = if ty.is_string() {
                        let mut segments = Punctuated::new();
                        segments.push(PathSegment {
                            ident: Ident::new("str", ty.span()),
                            arguments: syn::PathArguments::None,
                        });
                        let str_ty = Type::Path(TypePath {
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: segments.clone(),
                            },
                        });
                        (
                            str_ty.clone().into_reference("'a"),
                            str_ty.into_reference("'static"),
                        )
                    } else {
                        (
                            ty.clone().into_reference("'a"),
                            ty.into_reference("'static"),
                        )
                    };

                    ty_ref_el.push(ty_ref);
                    ty_static_el.push(ty_static);
                    idents.push(ident);
                });

                let ty = Type::Tuple(TypeTuple {
                    paren_token: Default::default(),
                    elems: ty_el,
                });
                let ty_ref = Type::Tuple(TypeTuple {
                    paren_token: Default::default(),
                    elems: ty_ref_el,
                });
                let ty_static = Type::Tuple(TypeTuple {
                    paren_token: Default::default(),
                    elems: ty_static_el,
                });

                (ty, ty_ref, ty_static, idents)
            }
        };

        (ty, ty_ref, ty_static, idents)
    }
}

/// General operations for `syn::Type`.
trait TypeOps {
    /// Check if the given `Type` is a `String`.
    fn is_string(&self) -> bool;

    /// Consume `self`, wrapping it in a reference.
    fn into_reference(self, symbol: &str) -> Type;
}

impl TypeOps for syn::Type {
    fn is_string(&self) -> bool {
        let mut segments: Punctuated<PathSegment, PathSep> = Punctuated::new();
        segments.push(PathSegment {
            ident: Ident::new("String", self.span()),
            arguments: syn::PathArguments::None,
        });

        *self
            == Type::Path(TypePath {
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments,
                },
            })
    }

    fn into_reference(self, symbol: &str) -> Type {
        Type::Reference(TypeReference {
            and_token: Token![&](self.span()),
            lifetime: Some(Lifetime::new(symbol, self.span())),
            mutability: None,
            elem: Box::new(self),
        })
    }
}
