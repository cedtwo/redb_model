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
        #[model(table_type = "multimap")]
        struct UnnamedModel;

        #[derive(Model)]
        #[model(table_type = "multimap", name = "named_model")]
        struct NamedModel;

        assert_eq!(NamedModel::DEFINITION.name(), "named_model");
        assert_eq!(UnnamedModel::DEFINITION.name(), "UnnamedModel");
    }

    #[test]
    fn test_type_alias() {
        #[derive(Model)]
        #[model(impl_ext)]
        struct TestModel {
            #[entry(
                position = "key",
                redb_type = "u8",
                from = "k as u32",
                into = "*k as u8"
            )]
            k: u32,
            #[entry(
                position = "value",
                redb_type = "u32",
                from = "v as u8",
                into = "*v as u32"
            )]
            v: u8,
        }

        assert_eq!(10 as <TestModel as ModelExt>::ModelKey, 10u32);
        assert_eq!(10 as <TestModel as ModelExt>::ModelValue, 10u8);
        assert_eq!(10 as <TestModel as ModelExt>::RedbKey, 10u8);
        assert_eq!(10 as <TestModel as ModelExt>::RedbValue, 10u32);
    }
}
