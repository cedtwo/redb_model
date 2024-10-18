use proc_macro::TokenStream;
use quote::quote;

use crate::{model, var};

/// Implement `Model`.
pub(super) fn impl_model(
    m: &model::ModelMeta,
    k: &var::ValueMeta,
    v: &var::ValueMeta,
) -> TokenStream {
    let m_ident = m.ident();
    let m_name = m.name();

    let k_ty = k.redb_ty();
    let v_ty = v.redb_ty();
    let m_ty = m.redb_ty(&k_ty, &v_ty);

    quote! {
        #[automatically_derived]
        impl<'a> Model<'a> for #m_ident {
                type TableType = #m_ty;
                const DEFINITION: Self::TableType = <#m_ty>::new(#m_name);
        }
    }
    .into()
}
