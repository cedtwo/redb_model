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
    let k_ty = k.redb_ty("'static");
    let v_ty = v.redb_ty("'static");
    let t_ty = t.redb_ty(&k_ty, &v_ty);

    let t_ident = t.ident();
    let t_name = t.name();

    quote! {
        #[automatically_derived]
        impl<'a> Model<'a> for #t_ident {
                type TableType = #t_ty;
                const DEFINITION: Self::TableType = <#t_ty>::new(#t_name);
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
    let redb_alias = def_redb_alias(&k, &v);
    let model_alias = def_model_alias(&k, &v);
    let model_ident = t.ident();

    let from_values = def_from_values(&model_ident, &k, &v);
    let from_guards = def_from_guards(&model_ident, &k, &v);
    let from_key_and_guard = def_from_key_and_guard(&model_ident, &k, &v);
    let as_values = def_as_values(&k, &v);
    let into_values = def_into_values(&k, &v);
    let clone_key = def_clone_key(&k);
    let clone_value = def_clone_value(&v);

    quote! {
        #[automatically_derived]
        impl<'a> redb_model::ModelExt<'a> for #model_ident {
            #redb_alias
            #model_alias

            #from_values
            #from_guards
            #from_key_and_guard
            #as_values
            #into_values
            #clone_key
            #clone_value
        }
    }
    .into()
}

/// Define the `RedbKey` and `RedbValue` alias's.
fn def_redb_alias(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
) -> proc_macro2::TokenStream {
    let k_ty = k.redb_ty("'static");
    let v_ty = v.redb_ty("'static");

    quote! {
        type RedbKey = #k_ty;
        type RedbValue = #v_ty;
    }
}

/// Define the `ModelKey` and `ModelValue` alias's.
fn def_model_alias(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
) -> proc_macro2::TokenStream {
    let k_ty = k.model_ty();
    let v_ty = v.model_ty();

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
    let k_ty = k.model_ty();
    let v_ty = v.model_ty();

    let k_ident = k.idents(None, None);
    let v_ident = v.idents(None, None);

    let all_idents = k.idents_flat().iter().chain(v.idents_flat());

    quote! {
        fn from_values(values: (Self::ModelKey, Self::ModelValue)) -> Self {
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
    let k_ident = k.idents(None, None);
    let v_ident = v.idents(None, None);

    let all_idents = k.idents_flat().iter().chain(v.idents_flat());

    quote! {
        fn from_guards(values: (&redb::AccessGuard<'a, Self::RedbKey>, &redb::AccessGuard<'a, Self::RedbValue>)) -> Self {
            let (#k_ident, #v_ident) = (values.0.value(), values.1.value());
            #t_ident {
                #( #all_idents: #all_idents.to_owned() ), *
            }
        }
    }
}

/// Define the `Model::from_key_and_guard` method.
fn def_from_key_and_guard(
    t_ident: &Ident,
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
) -> proc_macro2::TokenStream {
    let k_ident = k.idents(None, None);
    let v_ident = v.idents(None, None);

    let k_idents = k.idents_flat();
    let v_idents = v.idents_flat();

    quote! {
        fn from_key_and_guard(values: (Self::ModelKey, &redb::AccessGuard<'a, Self::RedbValue>)) -> Self {
            let (#k_ident, #v_ident) = (values.0, values.1.value());
            #t_ident {
                #( #k_idents ), *,
                #( #v_idents: #v_idents.to_owned() ), *
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

    let k_ty_ref = k.redb_ty("'a");
    let v_ty_ref = v.redb_ty("'a");

    quote! {
        fn as_values (&'a self) -> (#k_ty_ref, #v_ty_ref) {
            (#k_ident_ref, #v_ident_ref)
        }
    }
}

/// Define the `Model::into_values` method.
fn def_into_values(
    k: &var::CompositeVariable,
    v: &var::CompositeVariable,
) -> proc_macro2::TokenStream {
    let k_ty = k.model_ty();
    let v_ty = v.model_ty();

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
    let k_ty = k.model_ty();
    let k_ident_to_owned = k.idents(Some(quote!(self.)), Some(quote!(.to_owned())));

    quote! {
        fn clone_key (&self) -> (#k_ty) {
            #k_ident_to_owned
        }
    }
}

/// Define the `Model::clone_key` method.
fn def_clone_value(v: &var::CompositeVariable) -> proc_macro2::TokenStream {
    let v_ty = v.model_ty();
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
    let k_ty = k.model_ty();
    let v_ty = v.model_ty();

    quote! {
        #[automatically_derived]
        impl From<(#k_ty, #v_ty)> for #t_ident {
            fn from(values: (#k_ty, #v_ty)) -> Self {
                <Self as redb_model::ModelExt>::from_values(values)
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
    let generic_k = k.redb_ty("'static");
    let generic_v = v.redb_ty("'static");

    quote! {
        #[automatically_derived]
        impl<'a> From<(&redb::AccessGuard<'a, #generic_k>, &redb::AccessGuard<'a, #generic_v>)> for #t_ident {
            fn from(guards: (&redb::AccessGuard<'a, #generic_k>, &redb::AccessGuard<'a, #generic_v>)) -> Self {
                <Self as redb_model::ModelExt>::from_guards(guards)
            }
        }
    }
}
