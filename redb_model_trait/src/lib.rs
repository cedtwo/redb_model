/// Defines a table, and provides helper functions for table entries.
pub trait Model<'a, K: redb::Key + 'static, V: redb::Value + 'static>: Sized + 'a {
    /// The owned key type(s) held by an instance of the `Model` type.
    type ModelKey;
    /// The owned value type(s) held by an instance of the `Model` type.
    type ModelValue;

    /// The table definition.
    const DEFINITION: redb::TableDefinition<'a, K, V>;

    /// Return a reference to the `(Key, Value)` pair.
    fn as_values(
        &'a self,
    ) -> (
        <K as redb::Value>::SelfType<'a>,
        <V as redb::Value>::SelfType<'a>,
    );

    /// Consume the entry, returning a `(Key, Value)` pair.
    fn into_values(self) -> (Self::ModelKey, Self::ModelValue);

    /// Instantiate from a `(Key, Value)` pair.
    fn from_values(values: (Self::ModelKey, Self::ModelValue)) -> Self;

    /// Instantiate from an `AccessGuard` pair. Calls `to_owned` on variables.
    fn from_guards(values: (redb::AccessGuard<'a, K>, redb::AccessGuard<'a, V>)) -> Self;

    /// Clone the inner key.
    fn clone_key(&self) -> Self::ModelKey;

    /// Get the inner value.
    fn clone_value(&self) -> Self::ModelValue;
}
