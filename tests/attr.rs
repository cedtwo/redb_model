//! Attribute tests.
use redb::{MultimapTableHandle, TableHandle};
use redb_model::{Model, ModelExt};

#[cfg(test)]
mod tests {

    use super::*;

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
    fn test_multimap_table_name() {
        #[derive(Model)]
        #[model(table_type(multimap_table))]
        struct UnnamedModel;

        #[derive(Model)]
        #[model(table_type(multimap_table))]
        #[model(name = "named_model")]
        struct NamedModel;

        assert_eq!(NamedModel::DEFINITION.name(), "named_model");
        assert_eq!(UnnamedModel::DEFINITION.name(), "UnnamedModel");
    }

    #[test]
    fn test_impl_from() {
        #[derive(Model)]
        #[model(impl_ext, impl_from)]
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
    fn test_multimap_impl_from() {
        #[derive(Model)]
        #[model(table_type(multimap_table))]
        #[model(impl_ext, impl_from)]
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
        #[model(impl_ext)]
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
    fn test_multimap_distinct_value() {
        #[derive(Model)]
        #[model(table_type(multimap_table), impl_ext)]
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
        #[model(impl_ext)]
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

    #[test]
    fn test_multimap_composite_value() {
        #[derive(Model)]
        #[model(table_type(multimap_table), impl_ext)]
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
