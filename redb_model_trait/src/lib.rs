/// Trait for table definition.
pub trait Model<D> {
    /// The table definition.
    const DEFINITION: D;
}

/// Extension traits for entries derived from a `Model`.
pub trait ModelExt<'a, D, K, V>: Model<D> + Sized + 'a
where
    K: redb::Key + 'static,
    V: redb::Value + 'static,
{
    /// The owned key type(s) held by an instance of the `Model` type.
    type ModelKey;

    /// The owned value type(s) held by an instance of the `Model` type.
    type ModelValue;

    /// Instantiate from a `(Key, Value)` pair.
    fn from_values(values: (Self::ModelKey, Self::ModelValue)) -> Self;

    /// Instantiate from an `AccessGuard` pair. Calls `to_owned` on variables.
    fn from_guards(values: (redb::AccessGuard<'a, K>, redb::AccessGuard<'a, V>)) -> Self;

    /// Return a reference to the `(Key, Value)` pair.
    fn as_values(
        &'a self,
    ) -> (
        <K as redb::Value>::SelfType<'a>,
        <V as redb::Value>::SelfType<'a>,
    );

    /// Consume the entry, returning a `(Key, Value)` pair.
    fn into_values(self) -> (Self::ModelKey, Self::ModelValue);

    /// Clone the inner key.
    fn clone_key(&self) -> Self::ModelKey;

    /// Get the inner value.
    fn clone_value(&self) -> Self::ModelValue;
}
