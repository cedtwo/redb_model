# Redb Model
A derive macro for generating [`redb`] table definitions and (optionally) DTO object
conversion methods/implementations.

## Functionality

At a minimum, deriving `Model` on a named `struct` will implement the [`Model`]
trait, declaring `redb::TableDefinition` as an associated constant.

```rust
#[derive(Model)]
struct User {
    #[entry(position(key))]
    id: u32,
    #[entry(position(value))]
    name: String,
    #[entry(position(value))]
    email: String,
}

assert_eq!(User::DEFINITION.name(), "User");
```

In the example below, we specify the table name as "outbound_edge", and
generate an implementation of [`ModelExt`], providing common `DTO` functionality.

```rust
#
#[derive(Model)]
#[model(name = "outbound_edge", impl_ext)]
struct Edge {
    #[entry(position(key))]
    source: u32,
    #[entry(position(key))]
    target: u32,
    #[entry(position(value))]
    label: String,
}

assert_eq!(Edge::DEFINITION.name(), "outbound_edge");
// ModelExt::from_values
let edge = Edge::from_values(((0, 1), "label".to_owned()));

let txn = db.begin_write().unwrap();
{
    let mut table = txn.open_table(Edge::DEFINITION).unwrap();
    // ModelExt::as_values
    let (k, v) = edge.as_values();
    table.insert(k, v).unwrap();
}
txn.commit().unwrap();
```

## Struct Attributes

A model can be customized with the `model` attribute, providing any of the
following arguments:

Argument | Description | Type | Default
---|---|---|---
`table_name` | The table name passed to the definition | `String` | `<Struct name>` (case-sensitive)
`table_type` | Table type, either `table` or `multimap_table` | `String` | `table`
`impl_ext` | Implement [`ModelExt`] for the type | `bool` | `false`
`impl_from` | Implement `From<T>`, mapping `T` to the `from_values(T)` and `from_guards(T)` methods of [`ModelExt`]. Requires `impl_ext`. | `bool` | `false`

## Field Attributes

Values can be customized with the `entry` attribute. Each field must be specified
as either `key` or `value` with the `position` argument. Note that composite
variables (multiple `key` or `value` fields) are combined as tuples in the order
they are defined in the struct.

Argument | Description | Type | Default
---|---|---|---
`position` | The position of the field in an entry, either a `key` or a `value`. | `String` | N/A

## Implementation details

General notes regarding trait and type generation.

## Type Aliases

As of version `0.9.0`, generic arguments have been removed from [`Model`] and [`ModelExt`]
in exchange for type aliases. This decision was made to simplify trait definitions.
Take for example the trait below, where we avoid specifying any concrete types;

```rust
#[derive(Model)]
#[model(impl_ext)]
struct User {
    #[entry(position(key))]
    user_id: [u8; 16],
}

#[derive(Model)]
#[model(impl_ext)]
struct Article {
    #[entry(position(key))]
    article_id: [u8; 16],
    #[entry(position(value))]
    author_id: [u8; 16],
}

/// Get the key of model `T` from the implementing model.
trait SharedKey<'a, T: ModelExt<'a>>: ModelExt<'a> {
    /// Consume the instance, returning the key of model `T`.
    fn into_shared_key(self) -> T::ModelKey;
    /// Get a reference to the key of model `T`.
    fn shared_key_ref(&'a self) -> <T::RedbKey as redb::Value>::SelfType<'a>;
}


impl<'a> SharedKey<'a, User> for Article {
    fn into_shared_key(self) -> <User as ModelExt<'a>>::ModelKey {
        self.author_id
    }

    fn shared_key_ref(&'a self) -> <<User as ModelExt>::RedbKey as redb::Value>::SelfType<'a> {
        &self.author_id
    }
}

let article = Article::from_values(([0u8; 16], [0u8; 16]));
// Get the user id from an article.
let user_id = article.shared_key_ref();
// or
let user_id = SharedKey::<User>::shared_key_ref(&article);
```

### `String` Table Definitions

`redb_model` will replace `String` with `&str` for table definitions. This
is so that composite (tuple) variables can be borrowed and passed to database
handlers without destructuring the DTO.

### Unit type values

The unit type `()` must be passed if no value is defined.

```rust
#[derive(Model)]
#[model(name = "outbound_edge", impl_ext)]
struct Edge {
    #[entry(position(key))]
    source: [u8; 16],
    #[entry(position(key))]
    target: [u8; 16],
}
let k = ([0; 16], [1; 16]);
let v = ();
let e = Edge::from_values((k, v));
```

License: MIT OR Apache-2.0
