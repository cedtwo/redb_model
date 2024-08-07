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

#[cfg(test)]
mod tests {

    use crate::Model;
    use redb::TableHandle;

    #[test]
    fn test_table_name() {
        #[derive(Model)]
        struct UnnamedModel;

        #[derive(Model)]
        #[model(name = "named_model")]
        struct NamedModel;

        assert_eq!(NamedModel::DEFINITION.name(), "named_model");
        assert_eq!(UnnamedModel::DEFINITION.name(), "UnnamedModel");
    }

    #[test]
    fn test_impl_from() {
        #[derive(Model)]
        #[model(impl_from)]
        struct DistinctValueModel {
            #[entry(position(key))]
            key: [u8; 16],
            #[entry(position(value))]
            value: String,
        }

        let k_0 = [0; 16];
        let v_0 = String::from("Test String");
        let entry: DistinctValueModel = (k_0, v_0.clone()).into();
        let (k_1, v_1) = entry.into_values();

        assert_eq!(k_0, k_1);
        assert_eq!(v_0, v_1);
        // Failure is a compilation failure.
    }

    #[test]
    fn test_distinct_value() {
        #[derive(Model)]
        struct DistinctValueModel {
            #[entry(position(key))]
            key: [u8; 16],
            #[entry(position(value))]
            value: String,
        }

        let k_0 = [0; 16];
        let v_0 = String::from("Test String");
        let entry = DistinctValueModel::from_values((k_0, v_0.clone()));
        let (k_1, v_1) = entry.into_values();

        assert_eq!(k_0, k_1);
        assert_eq!(v_0, v_1);
        // Failure is a compilation failure.
    }

    #[test]
    fn test_composite_value() {
        #[derive(Model)]
        struct CompositeValueModel {
            #[entry(position(key))]
            key_0: [u8; 16],
            #[entry(position(key))]
            key_1: [u8; 16],
            #[entry(position(value))]
            value_0: String,
            #[entry(position(value))]
            value_1: String,
            #[entry(position(value))]
            value_2: String,
        }

        let k_0 = ([0; 16], [1; 16]);
        let v_0 = (
            String::from("S_0"),
            String::from("S_1"),
            String::from("S_2"),
        );
        let entry = CompositeValueModel::from_values((k_0, v_0.clone()));
        let (k_1, v_1) = entry.into_values();

        assert_eq!(k_0, k_1);
        assert_eq!(v_0, v_1);
        // Failure is a compilation failure.
    }
}
