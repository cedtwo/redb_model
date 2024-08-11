use args::VariablePosition;
use darling::{util::Shape, FromDeriveInput};
use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;
use syn::{parse_macro_input, DeriveInput};

mod args;
mod var;

/// Unwraps a `Result<T, darling::Error>`, or returns the error as a token stream.
macro_rules! unwrap_token_stream {
    (
        $expr:expr
    ) => {
        match $expr {
            Ok(args) => args,
            Err(e) => return TokenStream::from(e.write_errors()),
        }
    };
}

#[proc_macro_derive(Model, attributes(model, entry))]
#[allow(non_snake_case)]
pub fn Model(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_args = unwrap_token_stream!(args::ModelArgs::from_derive_input(&input));
    let fields = unwrap_token_stream!(struct_args.data.take_struct().ok_or(
        darling::Error::unsupported_shape_with_expected("Unnamed struct", &Shape::Named)
    ));

    let model_ident = struct_args.ident;
    let model_name = struct_args.name.unwrap_or_else(|| model_ident.to_string());

    let (k_fields, v_fields): (Vec<_>, Vec<_>) = fields
        .into_iter()
        .partition(|field| field.position == VariablePosition::Key);

    let k = unwrap_token_stream!(var::CompositeVariable::new(k_fields));
    let v = unwrap_token_stream!(var::CompositeVariable::new(v_fields));

    let mut stream = TokenStream::new();

    stream.extend(impl_model(&k, &v, &model_ident, model_name));

    if let Some(true) = struct_args.impl_from {
        stream.extend(impl_from(&k, &v, &model_ident));
    }

    stream
}

/// Implement `Model`, types and methods.
fn impl_model(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
    model_ident: &Ident,
    model_name: String,
) -> TokenStream {
    let k_ty_static = k.ty_static();
    let v_ty_static = v.ty_static();

    let model_alias = def_model_alias(&k, &v);
    let table_def = def_table_def(&k, &v, model_name);
    let as_values = def_as_values(&k, &v);
    let from_values = def_from_values(&model_ident, &k, &v);
    let from_guards = def_from_guards(&model_ident, &k, &v);
    let into_values = def_into_values(&k, &v);

    quote! {
        #[automatically_derived]
        impl<'a> Model<'a, #k_ty_static, #v_ty_static> for #model_ident {
            #model_alias

            #table_def

            #as_values
            #from_values
            #from_guards
            #into_values
        }
    }
    .into()
}

/// Define the `ModelKey` and `ModelValue` alias's.
fn def_model_alias(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
) -> proc_macro2::TokenStream {
    let k_ty = k.ty();
    let v_ty = v.ty();

    quote! {
        type ModelKey = #k_ty;
        type ModelValue = #v_ty;
    }
}

/// Define the `const` `redb::TableDefinition`.
fn def_table_def(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
    t_name: String,
) -> proc_macro2::TokenStream {
    let k_ty_static = k.ty_static();
    let v_ty_static = v.ty_static();

    quote! {
        const DEFINITION: redb::TableDefinition<'a, #k_ty_static, #v_ty_static> = redb::TableDefinition::new(#t_name);
    }
}

/// Define the `Model::into_values` method.
fn def_as_values(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
) -> proc_macro2::TokenStream {
    let k_ty_ref = k.ty_ref();
    let v_ty_ref = v.ty_ref();

    let k_ident = k.composite_ident(Some(quote! { &self. }));
    let v_ident = v.composite_ident(Some(quote! { &self. }));

    quote! {
        fn as_values (&'a self) -> (#k_ty_ref, #v_ty_ref) {
            (#k_ident, #v_ident)
        }
    }
}

/// Define the `Model::from_values` method.
fn def_from_values(
    t_ident: &Ident,
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
) -> proc_macro2::TokenStream {
    let k_ty = k.ty();
    let v_ty = v.ty();

    let k_ident = k.composite_ident(None);
    let v_ident = v.composite_ident(None);

    let all_idents = k.idents().iter().chain(v.idents());

    quote! {
        fn from_values(values: (#k_ty, #v_ty)) -> Self {
            let (#k_ident, #v_ident) = (values.0, values.1);
            #t_ident {
                #( #all_idents ), *
            }
        }
    }
}

/// Define the `Model::from_guards` method.
fn def_from_guards(
    t_ident: &Ident,
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
) -> proc_macro2::TokenStream {
    let k_ty_static = k.ty_static();
    let v_ty_static = v.ty_static();

    let k_ident = k.composite_ident(None);
    let v_ident = v.composite_ident(None);

    let all_idents = k.idents().iter().chain(v.idents());

    quote! {
        fn from_guards(guards: (redb::AccessGuard<'a, #k_ty_static>, redb::AccessGuard<'a, #v_ty_static>)) -> Self {
            let (#k_ident, #v_ident) = (guards.0.value(), guards.1.value());
            #t_ident {
                #( #all_idents: #all_idents.to_owned() ), *
            }
        }
    }
}

/// Define the `Model::into_values` method.
fn def_into_values(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
) -> proc_macro2::TokenStream {
    let k_ty = k.ty();
    let v_ty = v.ty();

    let k_scoped_ident = k.composite_ident(Some(quote! {self.}));
    let v_scoped_ident = v.composite_ident(Some(quote! {self.}));

    quote! {
        fn into_values(self) -> (#k_ty, #v_ty) {
            (#k_scoped_ident, #v_scoped_ident)
        }
    }
}

/// Implement `From<T>` for the given model, mapped to trait methods.
fn impl_from(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
    t_ident: &Ident,
) -> TokenStream {
    let from_values = impl_from_values(&k, &v, &t_ident);
    let from_guards = impl_from_guards(&k, &v, &t_ident);

    quote! {
        #from_values
        #from_guards
    }
    .into()
}

/// Implement `From<(K, V)>` for the given model.
fn impl_from_values(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
    t_ident: &Ident,
) -> proc_macro2::TokenStream {
    let k_ty = k.ty();
    let v_ty = v.ty();

    quote! {
        #[automatically_derived]
        impl From<(#k_ty, #v_ty)> for #t_ident {
            fn from(values: (#k_ty, #v_ty)) -> Self {
                Self::from_values(values)
            }
        }
    }
}

/// Implement `From<(redb::AccessGuard<'_, K>, AccessGuard<'_, V>)>` for the given model.
fn impl_from_guards(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
    t_ident: &Ident,
) -> proc_macro2::TokenStream {
    let k_ty_static = k.ty_static();
    let v_ty_static = v.ty_static();

    quote! {
        #[automatically_derived]
        impl<'a> From<(redb::AccessGuard<'a, #k_ty_static>, redb::AccessGuard<'a, #v_ty_static>)> for #t_ident {
            fn from(guards: (redb::AccessGuard<'a, #k_ty_static>, redb::AccessGuard<'a, #v_ty_static>)) -> Self {
                Self::from_guards(guards)
            }
        }
    }
}
