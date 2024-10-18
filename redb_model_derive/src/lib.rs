//! Derive macro for the `redb_model` crate.
use darling::{util::Shape, FromDeriveInput};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod args;
mod model;
mod var;

mod impl_from;
mod impl_model;
mod impl_model_ext;

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
    let fields_args = unwrap_token_stream!(struct_args.data.clone().take_struct().ok_or(
        darling::Error::unsupported_shape_with_expected("Unnamed struct", &Shape::Named)
    ));

    let impl_from = struct_args.impl_from;
    let impl_ext = struct_args.impl_ext;

    let (k_fields, v_fields): (Vec<_>, Vec<_>) = fields_args
        .into_iter()
        .partition(|field| *field.position() == args::EntryPosition::Key);

    // Model
    let m = model::ModelMeta::new(struct_args);
    // Key
    let k = var::ValueMeta::new(&k_fields);
    // Value
    let v = var::ValueMeta::new(&v_fields);

    let mut stream = TokenStream::new();

    // impl Model
    stream.extend(impl_model::impl_model(&m, &k, &v));
    // impl ModelExt
    if Some(true) == impl_ext {
        stream.extend(impl_model_ext::impl_model_ext(&m, &k, &v));
    }
    // impl From<T>
    if Some(true) == impl_from {
        if !(Some(true) == impl_ext) {
            return TokenStream::from(darling::Error::missing_field("impl_ext").write_errors());
        }
        stream.extend(impl_from::impl_from(&m, &k, &v));
    }

    stream
}
