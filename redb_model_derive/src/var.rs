//! Table variable interpolation.
use darling::util::Shape;
use proc_macro2::{Span, TokenStream};
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
    ty: Vec<Type>,
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

        let (ty, idents): (Vec<Type>, Vec<Ident>) = fields.into_iter().unzip();

        Ok(Self { ty, idents })
    }

    /// Get the type. or a tuple of the types.
    pub(crate) fn ty(&self) -> Type {
        match self.ty.len() {
            1 => self.ty[0].clone(),
            _ => {
                let mut ty_el = syn::punctuated::Punctuated::new();
                ty_el.extend(self.ty.clone());
                Type::Tuple(TypeTuple {
                    paren_token: Default::default(),
                    elems: ty_el,
                })
            }
        }
    }

    /// The `K` or `V` generic type value of the definition, either a type, or a
    /// a tuple of types. Leaves primitive numbers untouched. All other types
    /// are appended with the given lifetime. `String` is changed to `str`.
    pub(crate) fn types_as_generic(&self, lifetime: &str) -> Type {
        match self.ty.len() {
            1 => {
                let ty = self.ty[0].clone();
                if ty.is_string() {
                    let str_ty = Type::new_primitive("str", ty.span());
                    str_ty.into_reference(lifetime)
                } else if ty.is_primitive_number() {
                    ty.clone()
                } else {
                    ty.clone().into_reference(lifetime)
                }
            }
            _ => {
                let mut ty_el = syn::punctuated::Punctuated::new();

                self.ty.iter().for_each(|ty| {
                    ty_el.push(if ty.is_string() {
                        let str_ty = Type::new_primitive("str", ty.span());
                        str_ty.into_reference(lifetime)
                    } else if ty.is_primitive_number() {
                        ty.clone()
                    } else {
                        ty.clone().into_reference(lifetime)
                    });
                });

                Type::Tuple(TypeTuple {
                    paren_token: Default::default(),
                    elems: ty_el,
                })
            }
        }
    }

    /// Get the `Ident`, or a tuple of the `Ident`'s, optionally prefixing each variable.
    pub(crate) fn idents(
        &self,
        prefix: Option<proc_macro2::TokenStream>,
    ) -> proc_macro2::TokenStream {
        let prefix = prefix.unwrap_or(quote!());
        let idents = &self.idents;

        if idents.len() == 1 {
            quote! { #( #prefix #idents ) * }
        } else {
            quote! { ( #( #prefix #idents ), * ) }
        }
    }

    /// Get the vector of `Ident`s.
    pub(crate) fn idents_flat(&self) -> &Vec<Ident> {
        &self.idents
    }

    /// Return local `self.` prefixed variables. Returns as a reference `&` except
    /// where the type is a [`PRIMITIVE_NUMBER`].
    pub(crate) fn idents_as_ref(&self) -> TokenStream {
        match self.idents.len() {
            1 => {
                let ty = &self.ty[0];
                let ident = &self.idents[0];
                if ty.is_primitive_number() {
                    quote! { self. #ident }
                } else {
                    quote! { &self.  #ident }
                }
            }
            _ => {
                let mut tokens: Vec<TokenStream> = Vec::with_capacity(self.idents.len());

                self.ty.iter().zip(&self.idents).for_each(|(ty, ident)| {
                    tokens.push(if ty.is_primitive_number() {
                        quote! { self. #ident }
                    } else {
                        quote! { &self. #ident }
                    });
                });

                quote! { ( #( #tokens ), * ) }
            }
        }
    }
}

/// Primitive `Copy` types that do not implement `Value` for their `&'static` counterpart.
const PRIMITIVE_NUMBERS: [&str; 12] = [
    "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64",
];

/// General operations for `syn::Type`.
trait TypeOps {
    /// Create a new primitive type with no path.
    fn new_primitive(ty: &str, span: Span) -> Type;

    /// Check if the given `Type` is a `String`.
    fn is_string(&self) -> bool;

    /// Check if the given `Type` is a primitive number.
    fn is_primitive_number(&self) -> bool;

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

    fn is_primitive_number(&self) -> bool {
        if let Type::Path(TypePath {
            qself: None,
            path:
                Path {
                    leading_colon: None,
                    segments,
                },
        }) = self
        {
            if let Some(segment) = segments.last() {
                PRIMITIVE_NUMBERS.contains(&segment.ident.to_string().as_str())
            } else {
                false
            }
        } else {
            false
        }
    }

    fn into_reference(self, symbol: &str) -> Type {
        Type::Reference(TypeReference {
            and_token: Token![&](self.span()),
            lifetime: Some(Lifetime::new(symbol, self.span())),
            mutability: None,
            elem: Box::new(self),
        })
    }

    fn new_primitive(ty: &str, span: Span) -> Type {
        let mut segments = Punctuated::new();
        segments.push(PathSegment {
            ident: Ident::new(ty, span),
            arguments: syn::PathArguments::None,
        });
        Type::Path(TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: segments.clone(),
            },
        })
    }
}
