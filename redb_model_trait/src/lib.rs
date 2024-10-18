//! Traits for the `redb_model` crate.

/// Trait for table definition.
pub trait Model<'a> {
    /// The table type.
    type TableType;
    /// The table definition.
    const DEFINITION: Self::TableType;
}

/// Conversion methods for a `Model` and the associated keys and values.
pub trait ModelExt<'a>: Model<'a> + Sized + 'a {
    /// The `redb` definition key type(s).
    type RedbKey: redb::Key;
    /// The `redb` definition value type(s).
    type RedbValue: redb::Value;

    /// The model key type(s).
    type ModelKey;
    /// The model key type(s).
    type ModelValue;

    /// Instantiate from a `redb` (`K`, `V`) pair.
    fn from_values(
        values: (
            <Self::RedbKey as redb::Value>::SelfType<'a>,
            <Self::RedbValue as redb::Value>::SelfType<'a>,
        ),
    ) -> Self;

    /// Instantiate from a `redb` (`AccessGuard<K>`, `AccessGuard<V>`) pair.
    fn from_guards(
        values: (
            &redb::AccessGuard<'a, Self::RedbKey>,
            &redb::AccessGuard<'a, Self::RedbValue>,
        ),
    ) -> Self;

    /// Instantiate from a `redb` (`K`, `AccessGuard<V>`) pair.
    fn from_key_and_guard(
        values: (
            <Self::RedbKey as redb::Value>::SelfType<'a>,
            &redb::AccessGuard<'a, Self::RedbValue>,
        ),
    ) -> Self;

    /// Get the `redb` `K` value. May copy, clone or borrow
    /// depending on the definition.
    fn as_key(&'a self) -> <Self::RedbKey as redb::Value>::SelfType<'a>;

    /// Get the `redb` `V` value. May copy, clone or borrow
    /// depending on the definition.
    fn as_value(&'a self) -> <Self::RedbValue as redb::Value>::SelfType<'a>;

    /// Get all variables as a `redb` `(K, V)` pair. May copy, clone or borrow
    /// depending on the definition.
    fn as_key_and_value(
        &'a self,
    ) -> (
        <Self::RedbKey as redb::Value>::SelfType<'a>,
        <Self::RedbValue as redb::Value>::SelfType<'a>,
    );
}
