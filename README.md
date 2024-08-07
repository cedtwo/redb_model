# redb_model

## Redb Model
A derive macro for generating [`redb`] table definitions and DTO object
conversion methods/implementations.

### Functionality
All functionality is implemented by the [`Model`] trait within. Decorating
a struct with `#[derive(Model)]` will define a `redb::TableDefinition` as
an associated constant for the type, with the specified fields as key/value
types, or tuples of types.

#### Specifying keys and values
Key(s) and value(s) are specified by decorating table fields with
`#[entry(position(...))]`, passing either `key` or `value` to the inner field.
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
```

#### Specifying a table name
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

#### Type conversion
The generated implementation of the [`Model`] trait provides methods for
instantiating, borrowing and taking the key/value pairs from the model DTO.
See the methods of the [`Model`] trait for available methods. By default,
`#[derive(Model)]` will only generate an implementation of the `Model` trait.
Decorating the struct with `#[model(impl_from)]` will implement `From<T>`,
mapping `T` to the `from_values` and `from_guards` methods.
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
### Variable Ordering

All composite key/value variables are combined as a tuple in the order they
are defined.

License: MIT OR Apache-2.0
