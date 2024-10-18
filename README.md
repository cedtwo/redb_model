# Redb Model
A derive macro for generating [`redb`] table definitions and DTO object
conversion methods/implementations.

## Functionality

At a minimum, deriving `Model` on a named `struct` will implement the [`Model`]
trait, declaring `redb::TableDefinition` as an associated constant.

```rust
#[derive(Model)]
struct User {
    #[entry(position = "key")]
    id: u32,
    #[entry(position = "value")]
    name: String,
    #[entry(position = "value")]
    email: String,
}

// redb::TableDefinition::<u32, (String, String)>
assert_eq!(User::DEFINITION.name(), "User");
```

In the example below, the table name is specified as "outbound_edge", and the
`label` field is declared as `&str`, rather than `String` in the table. The
`impl_ext` argument on the struct generates an implementation of [`ModelExt`],
giving access to methods for type conversion between the DTO types and those
of the `redb` key/value. In this case, calling any of the `as_` methods will
`Copy` the `u32` fields, and borrow the `String` field as a `&str`.

```rust
#[derive(Model, Debug, PartialEq, Eq)]
#[model(name = "outbound_edge", impl_ext)]
struct Edge {
    #[entry(position = "key")]
    source: u32,
    #[entry(position = "key")]
    target: u32,
    #[entry(position = "value", redb_type = "&str")]
    label: String,
}

// redb::TableDefinition::<(u32, u32), &str>
assert_eq!(Edge::DEFINITION.name(), "outbound_edge");

let edge0 = Edge {
    source: 0,
    target: 1,
    label: String::from("label"),
};

let txn = db.begin_write().unwrap();
{
    // Model::DEFINITION
    let mut table = txn.open_table(Edge::DEFINITION).unwrap();
    // ModelExt::as_key_and_value
    let (k, v) = edge0.as_key_and_value();
    table.insert(k, v).unwrap();
}
txn.commit().unwrap();

// ModelExt::as_key
let k = edge0.as_key();
let txn = db.begin_read().unwrap();
// Model::DEFINITION
let table = txn.open_table(Edge::DEFINITION).unwrap();
let edge1 = table
    .get(k)
    .unwrap()
    // ModelExt::from_key_and_guard
    .map(|guard| Edge::from_key_and_guard((k, &guard)))
    .unwrap();

assert_eq!(edge0, edge1);
```

## Struct Attributes

A model can be customized with the `model` attribute, providing any of the
following arguments:

Argument | Description | Type | Default
---|---|---|---
`table_name` | The table name passed to the definition | `Literal` | `<struct name>` (case-sensitive)
`table_type` | Table type, either `table` or `multimap` | `Literal` | `table`
`impl_ext` | Implement [`ModelExt`] for the type | `bool` | `false`
`impl_from` | Implement `From<T>`, mapping `T` to `ModelExt::from_values(T)` and `ModelExt::from_guards(T)`. | `bool` | `false`

Note that `impl_from` uses methods of `impl_ext` and therefore requires both
arguments to be specified.

## Field Attributes

Values can be customized with the `entry` attribute. Each field must specify
`position` as `key` or `value`, and (optionally) provide an alternate `redb_type`.
When generating a `ModelExt` definition (providing `impl_ext` as a struct argument),
`from` and `into` operations may need to be explicit. Note that composite
variables (multiple `key` or `value` fields) are combined as tuples in the order
they are defined.

Argument | Description | Type | Default
---|---|---|---
`position` | The position of the field in an entry, either a `key` or a `value`. | `enum` (`key` or `value`) | `None`
`redb_type` | The type defined in the `redb::TableDefinition` or `redb::MultimapTableDefinition`. | `Type` | Field `Type`
`from` | The operation to convert **from** the `redb_type`.  | `Expression` | See below.
`into` | The operation to convert **into** the `redb_type`.  | `Expression` | See below.

Conversion `from` a `redb` value has the following default behavior (`impl_ext` only):
- If no `redb_type` is specified, the value is assumed to implement `Copy` and passed directly to the DTO.
- If the specified `redb_type` is a reference (is prefixed by `&`), `to_owned` is called on the value.
- If the specified `redb_type` is not a reference, `into` is called on the value.

Conversion `into` a `redb` value takes a type **reference**, and has the following default behavior (`impl_ext` only):
- If no `redb_type` is specified, the value is assumed to implement `Copy` and dereferenced.
- If the specified `redb_type` is a reference (is prefixed by `&`), the value is passed as a reference.
- If the specified `redb_type` is not a reference, `into` is called on the value.

For user defined types, typically implementing `From<RedbType> for FieldType` and
`Into<RedbType> for &FieldType` will satisfy type conversion.

```rust
#[derive(Copy, Clone)]
struct Wrapper(u32);

// Default `from` operation for `Wrapper`.
impl From<u32> for Wrapper {
    fn from(value: u32) -> Self {
        Wrapper(value)
    }
}

// Default `into` operation for `&Wrapper`.
impl Into<u32> for &Wrapper {
    fn into(self) -> u32 {
        self.0
    }
}

#[derive(Model)]
#[model(impl_ext)]
struct MyModel {
    #[entry(position = "key", redb_type = "u32")]
    key3: Wrapper
}
```

For external types where a *newtype* wrapper is not desired, or where the type
does not implement `Copy`, the `from` and `into` operations must be explicit.
Both the `from` and `into` argument accept a variable named after the field. For
`from` expressions, this argument is the `redb_type`, while for `into` expressions,
this is a **reference** of the field value. Note that while the operations are
named `from` and `into`, there is no constraint on what operations can be used,
as is demonstrated below.

```rust
use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

#[derive(Model)]
#[model(impl_ext)]
struct SecretModel {
    #[entry(
        position = "key",
        redb_type = "[u8; 16]",
        from = "Uuid::from_bytes(id)", // `id` is the `redb_type` (`[u8; 16]`).
        into = "id.into_bytes()" // `id` is `&Uuid`.
    )]
    id: Uuid,
    #[entry(
        position = "value",
        redb_type = "&str",
        from = "SecretString::from(secret)", // `secret` is the `redb_type` (`&str`).
        into = "secret.expose_secret()" // `secret` is `&SecretString`.
    )]
    secret: SecretString,
}
```

## Type Aliases

Generated definitions of the [`ModelExt`] traits defines type aliases for the
common key and value tuples used extensively in generated code. These may be
useful for extending the functionality of models without explicitly stating
key/value types.

Alias | Description
---|---
`ModelExt::RedbKey` | The `K` argument for the given `redb` definition.
`ModelExt::RedbValue` | The `V` argument for the given `redb` definition.
`ModelExt::ModelKey` | A tuple of the owned key type(s) defined in the model.
`ModelExt::ModelValue` | A tuple of the owned value type(s) defined in the model.


License: MIT OR Apache-2.0
