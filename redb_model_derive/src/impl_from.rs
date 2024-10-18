use proc_macro::TokenStream;
use quote::quote;

use crate::{model, var};

/// Implement `From<T>` for the given model, mapped to trait methods.
pub(super) fn impl_from(
    m: &model::ModelMeta,
    k: &var::ValueMeta,
    v: &var::ValueMeta,
) -> TokenStream {
    let from_values = impl_from_values(&m, &k, &v);
    let from_guards = impl_from_guards(&m, &k, &v);

    quote! {
        #from_values
        #from_guards
    }
    .into()
}

/// Implement `From<(K, V)>`.
fn impl_from_values(
    m: &model::ModelMeta,
    k: &var::ValueMeta,
    v: &var::ValueMeta,
) -> proc_macro2::TokenStream {
    let m_ident = m.ident();
    let k_redb_ty = k.redb_ty();
    let v_redb_ty = v.redb_ty();

    quote! {
        #[automatically_derived]
        impl From<(#k_redb_ty, #v_redb_ty)> for #m_ident {
            fn from(values: (#k_redb_ty, #v_redb_ty)) -> Self {
                <Self as redb_model::ModelExt>::from_values(values)
            }
        }
    }
}

/// Implement `From<(redb::AccessGuard<'_, K>, AccessGuard<'_, V>)>`.
fn impl_from_guards(
    m: &model::ModelMeta,
    k: &var::ValueMeta,
    v: &var::ValueMeta,
) -> proc_macro2::TokenStream {
    let m_ident = m.ident();
    let k_redb_ty = k.redb_ty();
    let v_redb_ty = v.redb_ty();

    quote! {
        #[automatically_derived]
        impl<'a> From<(&redb::AccessGuard<'a, #k_redb_ty>, &redb::AccessGuard<'a, #v_redb_ty>)> for #m_ident {
            fn from(guards: (&redb::AccessGuard<'a, #k_redb_ty>, &redb::AccessGuard<'a, #v_redb_ty>)) -> Self {
                <Self as redb_model::ModelExt>::from_guards(guards)
            }
        }
    }
}
