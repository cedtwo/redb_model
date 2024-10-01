//! Traits for the `redb_model` crate.

/// Trait for table definition.
pub trait Model<'a> {
    /// The table type.
    type TableType;
    /// The table definition.
    const DEFINITION: Self::TableType;
}

/// Extension types and traits for entries derived from a `Model`.
pub trait ModelExt<'a>: Model<'a> + Sized + 'a {
    /// The `redb` definition key type.
    type RedbKey: redb::Key + 'static;
    /// The `redb` definition value type.
    type RedbValue: redb::Value + 'static;

    /// The owned key type(s) held by an instance of the `Model` type.
    type ModelKey;
    /// The owned value type(s) held by an instance of the `Model` type.
    type ModelValue;

    /// Instantiate from a `(Key, Value)` pair.
    fn from_values(values: (Self::ModelKey, Self::ModelValue)) -> Self;

    /// Instantiate from an `AccessGuard` pair. Calls `to_owned` on variables.
    fn from_guards(
        values: (
            &redb::AccessGuard<'a, Self::RedbKey>,
            &redb::AccessGuard<'a, Self::RedbValue>,
        ),
    ) -> Self;

    /// Instantiate from an owned key and an `AccessGuard` value.
    fn from_key_and_guard(
        values: (Self::ModelKey, &redb::AccessGuard<'a, Self::RedbValue>),
    ) -> Self;

    /// Return a reference to the `(Key, Value)` pair.
    fn as_values(
        &'a self,
    ) -> (
        <Self::RedbKey as redb::Value>::SelfType<'a>,
        <Self::RedbValue as redb::Value>::SelfType<'a>,
    );

    /// Consume the entry, returning a `(Key, Value)` pair.
    fn into_values(self) -> (Self::ModelKey, Self::ModelValue);

    /// Clone the inner key.
    fn clone_key(&self) -> Self::ModelKey;

    /// Get the inner value.
    fn clone_value(&self) -> Self::ModelValue;
}
