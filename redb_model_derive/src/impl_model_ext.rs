use proc_macro::TokenStream;
use quote::quote;

use crate::{model, var};

/// Implement `ModelExt`.
pub(super) fn impl_model_ext(
    m: &model::ModelMeta,
    k: &var::ValueMeta,
    v: &var::ValueMeta,
) -> TokenStream {
    let m_ident = m.ident();

    let redb_alias = def_redb_alias(&k, &v);
    let model_alias = def_model_alias(&k, &v);

    let from_values = def_from_values(&m, &k, &v);
    let from_guards = def_from_guards(&m, &k, &v);
    let from_key_and_guard = def_from_key_and_guard(&m, &k, &v);
    let as_key = def_as_key(&k);
    let as_value = def_as_value(&v);
    let as_values = def_as_key_and_value(&k, &v);

    quote! {
        #[automatically_derived]
        impl<'a> redb_model::ModelExt<'a> for #m_ident{
            #redb_alias
            #model_alias

            #from_values
            #from_guards
            #from_key_and_guard
            #as_key
            #as_value
            #as_values
        }
    }
    .into()
}

/// Define the `ModelExt::RedbKey` and `ModelExt::RedbValue`.
fn def_redb_alias(k: &var::ValueMeta, v: &var::ValueMeta) -> proc_macro2::TokenStream {
    let k_ty = k.redb_ty();
    let v_ty = v.redb_ty();

    quote! {
        type RedbKey = #k_ty;
        type RedbValue = #v_ty;
    }
}

/// Define the `ModelExt::ModelKey` and `ModelExt::ModelValue`.
fn def_model_alias(k: &var::ValueMeta, v: &var::ValueMeta) -> proc_macro2::TokenStream {
    let k_ty = k.model_ty();
    let v_ty = v.model_ty();

    quote! {
        type ModelKey = #k_ty;
        type ModelValue = #v_ty;
    }
}

/// Define the `Model::from_values` method.
fn def_from_values(
    m: &model::ModelMeta,
    k: &var::ValueMeta,
    v: &var::ValueMeta,
) -> proc_macro2::TokenStream {
    let k_redb_ty = k.redb_ty();
    let v_redb_ty = v.redb_ty();

    let m_ident = m.ident();

    let k_ident_tuple = k.composite_idents();
    let v_ident_tuple = v.composite_idents();

    let kv = var::ValueMeta::new_merged(k, v);
    let kv_idents = kv.idents();
    let kv_from_methods = kv.from_methods();

    quote! {
        fn from_values(
            values: (
                <#k_redb_ty as redb::Value>::SelfType<'a>,
                <#v_redb_ty as redb::Value>::SelfType<'a>
            )
        ) -> Self {
            let (#k_ident_tuple, #v_ident_tuple) = (values.0, values.1);
            #m_ident { #( #kv_idents: #kv_from_methods), * }
        }
    }
}

/// Define the `Model::from_guards` method.
fn def_from_guards(
    m: &model::ModelMeta,
    k: &var::ValueMeta,
    v: &var::ValueMeta,
) -> proc_macro2::TokenStream {
    let k_redb_ty = k.redb_ty();
    let v_redb_ty = v.redb_ty();

    let m_ident = m.ident();

    let k_ident_tuple = k.composite_idents();
    let v_ident_tuple = v.composite_idents();

    let kv = var::ValueMeta::new_merged(k, v);
    let kv_idents = kv.idents().collect::<Vec<_>>();
    let kv_from_methods = kv.from_methods();

    quote! {
        fn from_guards(values: (&redb::AccessGuard<'a, #k_redb_ty>, &redb::AccessGuard<'a, #v_redb_ty>)) -> Self {
            // Destructure key and values.
            let (#k_ident_tuple, #v_ident_tuple) = (values.0.value(), values.1.value());
            // Apply type conversion.
            let ( #( #kv_idents ), * ) = ( #( #kv_from_methods ), *);

            #m_ident {
                #( #kv_idents ), *
            }
        }
    }
}

/// Define the `Model::from_key_and_guard` method.
fn def_from_key_and_guard(
    m: &model::ModelMeta,
    k: &var::ValueMeta,
    v: &var::ValueMeta,
) -> proc_macro2::TokenStream {
    let m_ident = m.ident();
    let k_redb_ty = k.redb_ty();
    let v_redb_ty = v.redb_ty();

    let k_ident_tuple = k.composite_idents();
    let v_ident_tuple = v.composite_idents();

    let kv = var::ValueMeta::new_merged(k, v);
    let kv_idents = kv.idents().collect::<Vec<_>>();
    let kv_from_methods = kv.from_methods();

    quote! {
        fn from_key_and_guard(values:
                (
                    <#k_redb_ty as redb::Value>::SelfType<'a>,
                    &redb::AccessGuard<'a, #v_redb_ty>
                )
            ) -> Self {
            // Destructure values.
            let (#k_ident_tuple, #v_ident_tuple) = (values.0, values.1.value());
            // Apply type conversion.
            let ( #( #kv_idents ), * ) = ( #( #kv_from_methods ), *);

            #m_ident {
                #( #kv_idents ), *
            }
        }
    }
}

/// Define the `Model::as_key` method.
fn def_as_key(k: &var::ValueMeta) -> proc_macro2::TokenStream {
    let k_redb_ty = k.redb_ty();
    let k_idents = k.idents().collect::<Vec<_>>();
    let k_into_methods = k.into_methods();
    let k_ident_tuple = k.composite_idents();

    quote! {
        fn as_key (&'a self) -> <#k_redb_ty as redb::Value>::SelfType<'a> {
            // Destructure struct.
            let ( #( #k_idents ), * ) = ( #( &self.#k_idents ), *);
            // Apply type conversion.
            let ( #( #k_idents ), * ) = ( #( #k_into_methods ), *);

            #k_ident_tuple
        }
    }
}

/// Define the `Model::as_value` method.
fn def_as_value(v: &var::ValueMeta) -> proc_macro2::TokenStream {
    let v_redb_ty = v.redb_ty();
    let v_idents = v.idents().collect::<Vec<_>>();
    let v_into_methods = v.into_methods();
    let v_ident_tuple = v.composite_idents();

    quote! {
        fn as_value (&'a self) -> <#v_redb_ty as redb::Value>::SelfType<'a> {
            // Destructure struct.
            let ( #( #v_idents ), * ) = ( #( &self.#v_idents ), *);
            // Apply type conversion.
            let ( #( #v_idents ), * ) = ( #( #v_into_methods ), *);

            #v_ident_tuple
        }
    }
}
/// Define the `Model::as_key_and_value` method.
fn def_as_key_and_value(k: &var::ValueMeta, v: &var::ValueMeta) -> proc_macro2::TokenStream {
    let k_redb_ty = k.redb_ty();
    let v_redb_ty = v.redb_ty();

    let k_ident_tuple = k.composite_idents();
    let v_ident_tuple = v.composite_idents();

    let kv = var::ValueMeta::new_merged(k, v);
    let kv_idents = kv.idents().collect::<Vec<_>>();
    let kv_into_methods = kv.into_methods();

    quote! {
        fn as_key_and_value (&'a self) -> (
            <#k_redb_ty as redb::Value>::SelfType<'a>,
            <#v_redb_ty as redb::Value>::SelfType<'a>
        ) {
            // Destructure struct.
            let ( #( #kv_idents ), * ) = ( #( &self.#kv_idents ), *);
            // Apply type conversion.
            let ( #( #kv_idents ), * ) = ( #( #kv_into_methods ), *);
            (
                #k_ident_tuple,
                #v_ident_tuple
            )
        }
    }
}
