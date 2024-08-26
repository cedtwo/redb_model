//! Derive macro for the `redb_model` crate.

use args::{ModelTableType, VariablePosition};
use darling::{util::Shape, FromDeriveInput};
use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;
use syn::{parse_macro_input, DeriveInput};

mod args;
mod table;
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
    let model_ty = struct_args.table_type.unwrap_or(ModelTableType::default());

    let (k_fields, v_fields): (Vec<_>, Vec<_>) = fields
        .into_iter()
        .partition(|field| field.position == VariablePosition::Key);

    let t = table::TableMetadata::new(model_ident, model_name, model_ty);
    let k = unwrap_token_stream!(var::CompositeVariable::new(k_fields));
    let v = unwrap_token_stream!(var::CompositeVariable::new(v_fields));

    let mut stream = TokenStream::new();

    stream.extend(impl_model(&k, &v, &t));
    if Some(true) == struct_args.impl_ext {
        stream.extend(impl_model_ext(&k, &v, &t));
    }
    if Some(true) == struct_args.impl_from {
        if Some(false) == struct_args.impl_ext || None == struct_args.impl_ext {
            return TokenStream::from(darling::Error::missing_field("impl_ext").write_errors());
        }
        stream.extend(impl_from(&k, &v, &t));
    }

    stream
}

/// Implement `ModelDefinition`.
fn impl_model(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
    t: &table::TableMetadata,
) -> TokenStream {
    let generic_k = k.types_as_generic("'static");
    let generic_v = v.types_as_generic("'static");
    let generic_t = t.type_as_generic(&generic_k, &generic_v);

    let table_ident = t.ident();
    let table_name = t.name();

    quote! {
        #[automatically_derived]
        impl<'a> Model<#generic_t> for #table_ident {
                const DEFINITION: #generic_t = <#generic_t>::new(#table_name);
        }
    }
    .into()
}

/// Implement `Model`, types and methods.
fn impl_model_ext(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
    t: &table::TableMetadata,
) -> TokenStream {
    let model_alias = def_model_alias(&k, &v);
    let model_ident = t.ident();

    let from_values = def_from_values(&model_ident, &k, &v);
    let from_guards = def_from_guards(&model_ident, &k, &v);
    let as_values = def_as_values(&k, &v);
    let into_values = def_into_values(&k, &v);
    let clone_key = def_clone_key(&k);
    let clone_value = def_clone_value(&v);

    let generic_k = k.types_as_generic("'static");
    let generic_v = v.types_as_generic("'static");
    let generic_t = t.type_as_generic(&generic_k, &generic_v);

    quote! {
        #[automatically_derived]
        impl<'a> redb_model::ModelExt<'a, #generic_t, #generic_k, #generic_v> for #model_ident {
            #model_alias

            #from_values
            #from_guards
            #as_values
            #into_values
            #clone_key
            #clone_value
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

/// Define the `Model::from_values` method.
fn def_from_values(
    t_ident: &Ident,
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
) -> proc_macro2::TokenStream {
    let k_ty = k.ty();
    let v_ty = v.ty();

    let k_ident = k.idents(None, None);
    let v_ident = v.idents(None, None);

    let all_idents = k.idents_flat().iter().chain(v.idents_flat());

    quote! {
        fn from_values(values: (#k_ty, #v_ty)) -> Self {
            let (#k_ident, #v_ident) = (values.0, values.1);
            #t_ident { #( #all_idents ), * }
        }
    }
}

/// Define the `Model::from_guards` method.
fn def_from_guards(
    t_ident: &Ident,
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
) -> proc_macro2::TokenStream {
    let generic_k = k.types_as_generic("'static");
    let generic_v = v.types_as_generic("'static");

    let k_ident = k.idents(None, None);
    let v_ident = v.idents(None, None);

    let all_idents = k.idents_flat().iter().chain(v.idents_flat());

    quote! {
        fn from_guards(guards: (redb::AccessGuard<'a, #generic_k>, redb::AccessGuard<'a, #generic_v>)) -> Self {
            let (#k_ident, #v_ident) = (guards.0.value(), guards.1.value());
            #t_ident {
                #( #all_idents: #all_idents.to_owned() ), *
            }
        }
    }
}

/// Define the `Model::into_values` method.
fn def_as_values(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
) -> proc_macro2::TokenStream {
    let k_ident_ref = k.idents_as_ref();
    let v_ident_ref = v.idents_as_ref();

    let generic_k_ref = k.types_as_generic("'a");
    let generic_v_ref = v.types_as_generic("'a");

    quote! {
        fn as_values (&'a self) -> (#generic_k_ref, #generic_v_ref) {
            (#k_ident_ref, #v_ident_ref)
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

    let k_scoped_ident = k.idents(Some(quote! {self.}), None);
    let v_scoped_ident = v.idents(Some(quote! {self.}), None);

    quote! {
        fn into_values(self) -> (#k_ty, #v_ty) {
            (#k_scoped_ident, #v_scoped_ident)
        }
    }
}

/// Define the `Model::clone_key` method.
fn def_clone_key(k: &var::CompositeVariable) -> proc_macro2::TokenStream {
    let k_ty = k.ty();
    let k_ident_to_owned = k.idents(Some(quote!(self.)), Some(quote!(.to_owned())));

    quote! {
        fn clone_key (&self) -> (#k_ty) {
            #k_ident_to_owned
        }
    }
}

/// Define the `Model::clone_key` method.
fn def_clone_value(v: &var::CompositeVariable) -> proc_macro2::TokenStream {
    let v_ty = v.ty();
    let v_ident_to_owned = v.idents(Some(quote!(self.)), Some(quote!(.to_owned())));

    quote! {
        fn clone_value (&self) -> (#v_ty) {
            #v_ident_to_owned
        }
    }
}

/// Implement `From<T>` for the given model, mapped to trait methods.
fn impl_from(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
    t: &table::TableMetadata,
) -> TokenStream {
    let t_ident = t.ident();
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
                <Self as redb_model::ModelExt<_, _, _>>::from_values(values)
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
    let generic_k = k.types_as_generic("'static");
    let generic_v = v.types_as_generic("'static");

    quote! {
        #[automatically_derived]
        impl<'a> From<(redb::AccessGuard<'a, #generic_k>, redb::AccessGuard<'a, #generic_v>)> for #t_ident {
            fn from(guards: (redb::AccessGuard<'a, #generic_k>, redb::AccessGuard<'a, #generic_v>)) -> Self {
                <Self as redb_model::ModelExt<_, _, _>>::from_guards(guards)
            }
        }
    }
}
