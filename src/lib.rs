//! # Redb Model
//! A derive macro for generating [redb] table definitions and (optionally) DTO object
//! conversion methods/implementations.
//!
//! ## Functionality
//!
//! At a minimum, deriving `Model` on a named `struct` will implement the [Model]
//! trait, declaring `redb::TableDefinition` as an associated constant.
//!
//! ```rust
//! # use redb::TableHandle;
//! # use redb_model::{Model};
//! #[derive(Model)]
//! struct User {
//!     #[entry(position(key))]
//!     id: u32,
//!     #[entry(position(value))]
//!     name: String,
//!     #[entry(position(value))]
//!     email: String,
//! }
//!
//! assert_eq!(User::DEFINITION.name(), "User");
//! ```
//!
//! Tables can be customized with optional arguments, and an implementation of
//! [ModelExt] can be generated to provide common `DTO` functionality.
//!
//! ```
//! # use redb::{Database, TableHandle};
//! # use redb::backends::InMemoryBackend;
//! # use redb_model::{Model, ModelExt};
//! # let db = Database::builder()
//! #     .create_with_backend(InMemoryBackend::new())
//! #     .unwrap();
//! #
//! #[derive(Model)]
//! #[model(name = "outbound_edge", impl_ext)]
//! struct Edge {
//!     #[entry(position(key))]
//!     source: u32,
//!     #[entry(position(key))]
//!     target: u32,
//!     #[entry(position(value))]
//!     label: String,
//! }
//!
//! assert_eq!(Edge::DEFINITION.name(), "outbound_edge");
//! let edge = Edge::from_values(((0, 1), "label".to_owned()));
//!
//! let txn = db.begin_write().unwrap();
//! {
//!     let mut table = txn.open_table(Edge::DEFINITION).unwrap();
//!     let (k, v) = edge.as_values();
//!     table.insert(k, v).unwrap();
//! }
//! txn.commit().unwrap();
//! ```
//!
//! ## Struct Attributes
//!
//! A model can be customized with the `model` attribute, providing any of the
//! following arguments:
//!
//! Argument | Description | Type | Default
//! ---|---|---|---
//! `table_name` | The table name passed to the definition | `String` | `<Struct name>` (case-sensitive)
//! `table_type` | Table type, either `table` or `multimap_table` | `String` | `table`
//! `impl_ext` | Implement [`ModelExt`] for the type | `bool` | `false`
//! `impl_from` | Implement `From<T>`, mapping `T` to the `from_values(T)` and `from_guards(T)` methods of [`ModelExt`]. Requires `impl_ext`. | `bool` | `false`
//!
//! ## Field Attributes
//!
//! Values can be customized with the `entry` attribute. Each field must be specified
//! as either `key` or `value` with the `position` argument. Note that composite
//! variables (multiple `key` or `value` fields) are combined as tuples in the order
//! they are defined in the struct.
//!
//! Argument | Description | Type | Default
//! ---|---|---|---
//! `position` | The position of the field in an entry, either a `key` or a `value`. | `String` | N/A
//!
//! ## Implementation details
//!
//! General notes regarding trait and type generation.
//!
//! ### `String` Table Definitions
//!
//! `redb_model` will replace `String` with `&str` for table definitions. This
//! is so that composite (tuple) variables can be borrowed and passed to database
//! handlers without destructuring the DTO.
//!
//! ### Unit type values
//!
//! The unit type `()` must be passed if no value is defined.
//!
//! ```rust
//! # use redb::TableHandle;
//! # use redb_model::{Model, ModelExt};
//! #[derive(Model)]
//! #[model(name = "outbound_edge", impl_ext)]
//! struct Edge {
//!     #[entry(position(key))]
//!     source: [u8; 16],
//!     #[entry(position(key))]
//!     target: [u8; 16],
//! }
//! let k = ([0; 16], [1; 16]);
//! let v = ();
//! let e = Edge::from_values((k, v));
//! ```
pub use _derive::Model;
pub use _trait::{Model, ModelExt};
