# Redb Model
A derive macro for generating [`redb`] table definitions and DTO object
conversion methods/implementations.

## Functionality

This crate aims to unify database entry DTO definitions and `redb::TableDefinition`.
Decorating a struct with `#[derive(Model)]` will implement `Model` for the type,
define `redb::TableDefinition` as an associated constant, and generating helper
methods for the type.

### Example

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

let user = User {
    id: 0,
    name: "User".to_owned(),
    email: "user_email@email.com".to_owned(),
};

let txn = db.begin_write().unwrap();
{
    let mut table = txn.open_table(User::DEFINITION).unwrap();
    let (k, v) = user.as_values();
    table.insert(k, v).unwrap();
}
txn.commit().unwrap();
```

### Specifying keys and values
Key(s) and value(s) are specified by decorating table fields with
`#[entry(position(...))]`, passing either `key` or `value` to the inner field.
A composite key/value will be combined into a tuple.

```rust
#[derive(Model)]
struct User {
    #[entry(position(key))]
    uuid: [u8; 16],
    #[entry(position(value))]
    username: String,
    #[entry(position(value))]
    email: String,
}

let user_key = [0; 16];
let user_value = ("my_name".to_string(), "my_email@email.com".to_string());

let user = User::from_values((user_key, user_value));
let (k, v) = user.into_values();

// Only the value is a tuple for this model.
assert_eq!(k, [0; 16]);
assert_eq!(v, ("my_name".to_string(), "my_email@email.com".to_string()));
```

### Specifying a table name
Table names default to the (case-sensitive) struct name. This can be overridden by decorating
the struct with `#[model(name = "...")]` attribute.
```rust
#[derive(Model)]
#[model(name = "user_table")]
struct User {
    #[entry(position(key))]
    uuid: [u8; 16],
    #[entry(position(value))]
    username: String,
}

assert_eq!(User::DEFINITION.name(), "user_table");
```

### Type conversion
The generated implementation of the `Model` trait provides methods for
instantiating, borrowing and taking the key/value pairs of the model DTO.
Decorating the struct with `#[model(impl_from)]` will further implement
`From<T>`, mapping `T` to the `from_values(T)` and `from_guards(T)` methods.
```rust
#[derive(Model)]
#[model(impl_from)]
struct User {
    #[entry(position(key))]
    uuid: [u8; 16],
    #[entry(position(value))]
    username: String,
    #[entry(position(value))]
    email: String,
}

let user_key = [0; 16];
let user_value = ("my_name".to_string(), "my_email@email.com".to_string());

let user: User = ((user_key, user_value)).into();
```
## Implementation details

The following are general notes regarding implementation.

### `String` Table Definitions

`redb_model` will replace `String` with `&str` for table definitions. This
is so that composite (tuple) variables can be borrowed and passed to database
handlers without destructuring the DTO. This is not currently possible with
`String`.
```rust
const TABLE: TableDefinition<(String, String), ()> = TableDefinition::new("table");

let string_0 = "string_0".to_string();
let string_1 = "string_1".to_string();

let txn = db.begin_write().unwrap();
let mut table = txn.open_table(TABLE).unwrap();
// This doesn't work.
table.insert((&string_0, &string_1), ());
// Neither does this.
table.insert((string_0.as_str(), string_1.as_str()), ());
```

### Unit type values

The unit type `()` must be passed if no value is defined.
```rust
#[derive(Model)]
#[model(name = "outbound_edge")]
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

### Variable Ordering

All composite key/value variables are combined as a tuple in the order they
are defined. This is intended but can be changed if there is any reason
to do so.

### The `Model` definition and `redb::TableDefinition`

The `redb::TableDefinition` uses `'static` references of the types defined
in the `Model`, with the exception of `String` which uses a `'static` string
slice. This is to ensure that calling `as_values` returns references suitable
for database calls.

License: MIT OR Apache-2.0
