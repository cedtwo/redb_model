//! # Redb Model
//! A derive macro for generating [`redb`] table definitions and DTO object
//! conversion methods/implementations.
//!
//! ## Functionality
//! All functionality is implemented by the [`Model`] trait within. Decorating
//! a struct with `#[derive(Model)]` will define a `redb::TableDefinition` as
//! an associated constant for the type, with the specified fields as key/value
//! types, or tuples of types.
//!
//! ### Specifying keys and values
//! Key(s) and value(s) are specified by decorating table fields with
//! `#[entry(position(...))]`, passing either `key` or `value` to the inner field.
//! ```
//! # use crate::redb_model::Model;
//! #
//! #[derive(Model)]
//! struct User {
//!     #[entry(position(key))]
//!     uuid: [u8; 16],
//!     #[entry(position(value))]
//!     username: String,
//!     #[entry(position(value))]
//!     email: String,
//! }
//!
//! let user_key = [0; 16];
//! let user_value = ("my_name".to_string(), "my_email@email.com".to_string());
//!
//! let user = User::from_values((user_key, user_value));
//! ```
//!
//! ### Specifying a table name
//! Table names default to the (case-sensitive) struct name. This can be overridden by decorating
//! the struct with `#[model(name = "...")]` attribute.
//! ```
//! # use redb::TableHandle;
//! #
//! # use crate::redb_model::Model;
//! #
//! #[derive(Model)]
//! #[model(name = "user_table")]
//! struct User {
//!     #[entry(position(key))]
//!     uuid: [u8; 16],
//!     #[entry(position(value))]
//!     username: String,
//! }
//!
//! assert_eq!(User::DEFINITION.name(), "user_table");
//! ```
//!
//! ### Type conversion
//! The generated implementation of the [`Model`] trait provides methods for
//! instantiating, borrowing and taking the key/value pairs of the model DTO.
//! See the [`Model`] trait for available methods. By default, `#[derive(Model)]`
//! will only generate an implementation of the `Model` trait. Decorating the
//! struct with `#[model(impl_from)]` will implement `From<T>`, mapping `T` to
//! the `from_values(T)` method.
//! ```
//! # use crate::redb_model::Model;
//! #
//! #[derive(Model)]
//! #[model(impl_from)]
//! struct User {
//!     #[entry(position(key))]
//!     uuid: [u8; 16],
//!     #[entry(position(value))]
//!     username: String,
//!     #[entry(position(value))]
//!     email: String,
//! }
//!
//! let user_key = [0; 16];
//! let user_value = ("my_name".to_string(), "my_email@email.com".to_string());
//!
//! let user: User = ((user_key, user_value)).into();
//! ```
//! ## Implementation details
//!
//! The following are general notes regardign implementation or usage.
//!
//! ### Unit type values
//!
//! The unit type `()` must be passed if no value is defined.
//! ```
//! # use redb::TableHandle;
//! #
//! # use crate::redb_model::Model;
//! #
//! #[derive(Model)]
//! #[model(name = "outbound_edge")]
//! struct Edge {
//!     #[entry(position(key))]
//!     source: [u8; 16],
//!     #[entry(position(key))]
//!     target: [u8; 16],
//! }
//! let k = ([0; 16], [1; 16]);
//! let v = (); // `()` argument must be passed.
//! let e = Edge::from_values((k, v));
//! ```
//!
//! ### Variable Ordering
//!
//! All composite key/value variables are combined as a tuple in the order they
//! are defined. This is intended but can be changed if there is any reason
//! to do so.
//!
//! ### The `Model` definition and `redb::TableDefinition`
//!
//! The `redb::TableDefinition` uses `'static` references of the types defined
//! in the `Model`, with the exception of `String` which uses a `'static` string
//! slice. This is to ensure that calling `as_values` returns references suitable
//! for database calls.

pub use _derive::Model;
pub use _trait::Model;

#[cfg(test)]
mod tests {

    use crate::Model;
    use redb::TableHandle;

    #[test]
    fn test_table_name() {
        #[derive(Model)]
        struct UnnamedModel;

        #[derive(Model)]
        #[model(name = "named_model")]
        struct NamedModel;

        assert_eq!(NamedModel::DEFINITION.name(), "named_model");
        assert_eq!(UnnamedModel::DEFINITION.name(), "UnnamedModel");
    }

    #[test]
    fn test_impl_from() {
        #[derive(Model)]
        #[model(impl_from)]
        struct DistinctValueModel {
            #[entry(position(key))]
            key: [u8; 16],
            #[entry(position(value))]
            value: String,
        }

        let k_0 = [0; 16];
        let v_0 = String::from("Test String");
        let entry: DistinctValueModel = (k_0, v_0.clone()).into();
        let (k_1, v_1) = entry.into_values();

        assert_eq!(k_0, k_1);
        assert_eq!(v_0, v_1);
        // Failure is a compilation failure.
    }

    #[test]
    fn test_distinct_value() {
        #[derive(Model)]
        struct DistinctValueModel {
            #[entry(position(key))]
            key: [u8; 16],
            #[entry(position(value))]
            value: String,
        }

        let k_0 = [0; 16];
        let v_0 = String::from("Test String");
        let entry = DistinctValueModel::from_values((k_0, v_0.clone()));
        let (k_1, v_1) = entry.into_values();

        assert_eq!(k_0, k_1);
        assert_eq!(v_0, v_1);
        // Failure is a compilation failure.
    }

    #[test]
    fn test_composite_value() {
        #[derive(Model)]
        struct CompositeValueModel {
            #[entry(position(key))]
            key_0: [u8; 16],
            #[entry(position(key))]
            key_1: [u8; 16],
            #[entry(position(value))]
            value_0: String,
            #[entry(position(value))]
            value_1: String,
            #[entry(position(value))]
            value_2: String,
        }

        let k_0 = ([0; 16], [1; 16]);
        let v_0 = (
            String::from("S_0"),
            String::from("S_1"),
            String::from("S_2"),
        );
        let entry = CompositeValueModel::from_values((k_0, v_0.clone()));
        let (k_1, v_1) = entry.into_values();

        assert_eq!(k_0, k_1);
        assert_eq!(v_0, v_1);
        // Failure is a compilation failure.
    }
}
