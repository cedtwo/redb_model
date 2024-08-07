/// Defines a table, and provides helper functions for table entries.
pub trait Model<'a, K: redb::Key + 'static, V: redb::Value + 'static>: Sized + 'a {
    /// A key type reference, or a tuple of key type references.
    type KeyRef;
    /// A value type reference, or a tuple of value type references.
    type ValueRef;

    /// The table definition.
    const DEFINITION: redb::TableDefinition<'a, K, V>;

    /// Return a reference to the `(Key, Value)` pair, wrapped in a tuple for
    /// composite variables.
    fn as_values(&'a self) -> (Self::KeyRef, Self::ValueRef);

    /// Consume the entry, returning a `(Key, Value)` pair.
    fn into_values(self) -> (K, V);

    /// Instantiate from a `(Key, Value)` pair.
    fn from_values(values: (K, V)) -> Self;

    /// Instantiate from an `(AccessGuard<K>, AccessGuard<V>)` pair.
    fn from_guards(values: (redb::AccessGuard<'a, K>, redb::AccessGuard<'a, V>)) -> Self
    where
        K: for<'c> redb::Value<SelfType<'c> = K>,
        V: for<'c> redb::Value<SelfType<'c> = V>,
    {
        Self::from_values((values.0.value(), values.1.value()))
    }
}
