//! Type tests.
use redb::{backends::InMemoryBackend, Database, ReadableTableMetadata};

use _derive::Model;
use _trait::{Model, ModelExt};

#[macro_use]
// mod macros;
mod macros {
    /// Test both `ModelExt::from_values` and `From::from`.
    macro_rules! test_from_values {
        (
        $struct_name:ident,
        ( $k:ident, $v:ident )
    ) => {
            let _ = $struct_name::from_values(($k.clone(), $v.clone()));
            let _ = $struct_name::from(($k.clone(), $v.clone()));
        };
    }

    /// Test both `ModelExt::from_guards` and `From::from`.
    macro_rules! test_from_guards {
        (
        $database:ident,
        $struct_name:ident,
        ( $k:ident, $v:ident )
    ) => {
            // Insert an entry.
            let entry_in = $struct_name::from_values(($k.clone(), $v.clone()));
            let txn = $database.begin_write().unwrap();
            {
                let mut table = txn.open_table($struct_name::DEFINITION).unwrap();
                let (k, v) = entry_in.as_key_and_value();
                table.insert(k, v).unwrap();
            }
            txn.commit().unwrap();

            // Get all entries from the table.
            type KR<'a> =
                <<$struct_name as redb_model::ModelExt<'a>>::RedbKey as redb::Value>::SelfType<'a>;
            let txn = $database.begin_read().unwrap();
            let table = txn.open_table($struct_name::DEFINITION).unwrap();
            let (entry_out_0, entry_out_1) = table
                .range::<KR>(..)
                .unwrap()
                .map(|r| {
                    r.map(|(k_guard, v_guard)| {
                        (
                            $struct_name::from_guards((&k_guard, &v_guard)),
                            $struct_name::from((&k_guard, &v_guard)),
                        )
                    })
                })
                .next()
                .unwrap()
                .unwrap();

            assert_eq!(entry_in, entry_out_0);
            assert_eq!(entry_in, entry_out_1);

            $database
                .begin_write()
                .unwrap()
                .delete_table($struct_name::DEFINITION)
                .unwrap();
        };
    }

    /// Test `ModelExt::from_key_and_guard`.
    macro_rules! test_from_key_and_guard {
        (
        $database:ident,
        $struct_name:ident,
        ( $k:ident, $v:ident )
    ) => {
            // Insert an entry.
            let entry_in = $struct_name::from_values(($k.clone(), $v.clone()));
            let txn = $database.begin_write().unwrap();
            {
                let mut table = txn.open_table($struct_name::DEFINITION).unwrap();
                let (k, v) = entry_in.as_key_and_value();
                table.insert(k, v).unwrap();
            }
            txn.commit().unwrap();

            // Get the entry from the table.
            let txn = $database.begin_read().unwrap();
            let table = txn.open_table($struct_name::DEFINITION).unwrap();
            let entry_out = table
                .get($k.clone())
                .unwrap()
                .map(|v_guard| $struct_name::from_key_and_guard(($k.clone(), &v_guard)))
                .unwrap();

            assert_eq!(entry_in, entry_out);

            $database
                .begin_write()
                .unwrap()
                .delete_table($struct_name::DEFINITION)
                .unwrap();
        };
    }

    /// Test `ModelExt::as_values`.
    macro_rules! test_as_key_and_value {
        (
        $database:ident,
        $struct_name:ident,
        ( $k:ident, $v:ident )
    ) => {
            // Insert an entry.
            let entry = $struct_name::from_values(($k.clone(), $v.clone()));
            let txn = $database.begin_write().unwrap();
            {
                let mut table = txn.open_table($struct_name::DEFINITION).unwrap();
                let (k, v) = entry.as_key_and_value();
                table.insert(k, v).unwrap();
            }
            txn.commit().unwrap();

            // Assert the number of entries in the table.
            let txn = $database.begin_read().unwrap();
            let table = txn.open_table($struct_name::DEFINITION).unwrap();
            assert_eq!(table.len().unwrap(), 1);

            $database
                .begin_write()
                .unwrap()
                .delete_table($struct_name::DEFINITION)
                .unwrap();
        };
    }

    /// Test `ModelExt::clone_key`.
    macro_rules! test_as_key {
        (
        $struct_name:ident,
        ( $k:ident, $v:ident )
    ) => {
            let entry = $struct_name::from_values(($k.clone(), $v.clone()));
            assert_eq!($k, entry.as_key());
        };
    }

    /// Test `ModelExt::clone_value`.
    macro_rules! test_as_value {
        (
        $struct_name:ident,
        ( $k:ident, $v:ident )
    ) => {
            let entry = $struct_name::from_values(($k.clone(), $v.clone()));
            assert_eq!($v, entry.as_value());
        };
    }
}

#[test]
fn test_single_copy_type() {
    let db = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap();

    #[derive(Model, PartialEq, Eq, Debug)]
    #[model(impl_ext, impl_from)]
    struct SingleValue {
        #[entry(position(key))]
        key: u32,
        #[entry(position(value))]
        value: u32,
    }

    let (k, v) = (0, 1);

    test_from_values!(SingleValue, (k, v));
    test_from_guards!(db, SingleValue, (k, v));
    test_from_key_and_guard!(db, SingleValue, (k, v));
    test_as_key_and_value!(db, SingleValue, (k, v));
    test_as_key!(SingleValue, (k, v));
    test_as_value!(SingleValue, (k, v));
}

#[test]
fn test_composite_copy_type() {
    let db = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap();

    #[derive(Model, PartialEq, Eq, Debug)]
    #[model(impl_ext, impl_from)]
    struct CompositeValue {
        #[entry(position(key))]
        key_0: u32,
        #[entry(position(key))]
        key_1: u64,
        #[entry(position(value))]
        value0: u32,
        #[entry(position(value))]
        value1: u64,
    }

    let (k, v) = ((0, 1), (2, 3));

    test_from_values!(CompositeValue, (k, v));
    test_from_guards!(db, CompositeValue, (k, v));
    test_from_key_and_guard!(db, CompositeValue, (k, v));
    test_as_key_and_value!(db, CompositeValue, (k, v));
    test_as_key!(CompositeValue, (k, v));
    test_as_value!(CompositeValue, (k, v));
}

#[test]
fn test_single_borrow_type() {
    let db = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap();

    #[derive(Model, PartialEq, Eq, Debug)]
    #[model(impl_ext, impl_from)]
    struct SingleValue {
        #[entry(position = "key", redb_type = "&str", into = "key.as_str()")]
        key: String,
        #[entry(position = "value", redb_type = "&str", into = "value.as_str()")]
        value: String,
    }

    let (k, v) = ("key", "value");

    test_from_values!(SingleValue, (k, v));
    test_from_guards!(db, SingleValue, (k, v));
    test_from_key_and_guard!(db, SingleValue, (k, v));
    test_as_key_and_value!(db, SingleValue, (k, v));
    test_as_key!(SingleValue, (k, v));
    test_as_value!(SingleValue, (k, v));
}

#[test]
fn test_composite_borrow_type() {
    let db = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap();

    #[derive(Model, PartialEq, Eq, Debug)]
    #[model(impl_ext, impl_from)]
    struct CompositeValue {
        #[entry(position = "key", redb_type = "&str", into = "key0.as_str()")]
        key0: String,
        #[entry(position = "key", redb_type = "&str", into = "key1.as_str()")]
        key1: String,
        #[entry(position = "value", redb_type = "&str", into = "value0.as_str()")]
        value0: String,
        #[entry(position = "value", redb_type = "&str", into = "value1.as_str()")]
        value1: String,
    }

    let (k, v) = (("key0", "key1"), ("value0", "value1"));

    test_from_values!(CompositeValue, (k, v));
    test_from_guards!(db, CompositeValue, (k, v));
    test_from_key_and_guard!(db, CompositeValue, (k, v));
    test_as_key_and_value!(db, CompositeValue, (k, v));
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct TestWrapper(u32);

#[test]
fn test_single_wrapper_type() {
    let db = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap();

    #[derive(Model, PartialEq, Eq, Debug)]
    #[model(impl_ext, impl_from)]
    struct SingleValue {
        #[entry(
            position = "key",
            redb_type = "u32",
            from = "TestWrapper(key)",
            into = "key.0"
        )]
        key: TestWrapper,
        #[entry(
            position = "value",
            redb_type = "u32",
            from = "TestWrapper(value)",
            into = "value.0"
        )]
        value: TestWrapper,
    }

    let (k, v) = (0, 1);

    test_from_values!(SingleValue, (k, v));
    test_from_guards!(db, SingleValue, (k, v));
    test_from_key_and_guard!(db, SingleValue, (k, v));
    test_as_key_and_value!(db, SingleValue, (k, v));
}

#[test]
fn test_composite_wrapper_type() {
    let db = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap();

    #[derive(Model, PartialEq, Eq, Debug)]
    #[model(impl_ext, impl_from)]
    struct CompositeValue {
        #[entry(
            position = "key",
            redb_type = "u32",
            from = "TestWrapper(key0)",
            into = "key0.0"
        )]
        key0: TestWrapper,
        #[entry(
            position = "key",
            redb_type = "u32",
            from = "TestWrapper(key1)",
            into = "key1.0"
        )]
        key1: TestWrapper,
        #[entry(
            position = "value",
            redb_type = "u32",
            from = "TestWrapper(value0)",
            into = "value0.0"
        )]
        value0: TestWrapper,
        #[entry(
            position = "value",
            redb_type = "u32",
            from = "TestWrapper(value1)",
            into = "value1.0"
        )]
        value1: TestWrapper,
    }

    let (k, v) = ((0, 1), (2, 3));

    test_from_values!(CompositeValue, (k, v));
    test_from_guards!(db, CompositeValue, (k, v));
    test_from_key_and_guard!(db, CompositeValue, (k, v));
    test_as_key_and_value!(db, CompositeValue, (k, v));
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
struct TestWrapperImpl(u32);

impl From<u32> for TestWrapperImpl {
    fn from(value: u32) -> Self {
        TestWrapperImpl(value)
    }
}

impl Into<u32> for &TestWrapperImpl {
    fn into(self) -> u32 {
        self.0
    }
}

#[test]
fn test_single_wrapper_impl_type() {
    let db = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap();

    #[derive(Model, PartialEq, Eq, Debug)]
    #[model(impl_ext, impl_from)]
    struct SingleValue {
        #[entry(position = "key", redb_type = "u32")]
        key: TestWrapperImpl,
        #[entry(position = "value", redb_type = "u32")]
        value: TestWrapperImpl,
    }

    let (k, v) = (0, 1);

    test_from_values!(SingleValue, (k, v));
    test_from_guards!(db, SingleValue, (k, v));
    test_from_key_and_guard!(db, SingleValue, (k, v));
    test_as_key_and_value!(db, SingleValue, (k, v));
}

#[test]
fn test_composite_wrapper_impl_type() {
    let db = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap();

    #[derive(Model, PartialEq, Eq, Debug)]
    #[model(impl_ext, impl_from)]
    struct CompositeValue {
        #[entry(position = "key", redb_type = "u32")]
        key0: TestWrapperImpl,
        #[entry(position = "key", redb_type = "u32")]
        key1: TestWrapperImpl,
        #[entry(position = "value", redb_type = "u32")]
        value0: TestWrapperImpl,
        #[entry(position = "value", redb_type = "u32")]
        value1: TestWrapperImpl,
    }

    let (k, v) = ((0, 1), (2, 3));

    test_from_values!(CompositeValue, (k, v));
    test_from_guards!(db, CompositeValue, (k, v));
    test_from_key_and_guard!(db, CompositeValue, (k, v));
    test_as_key_and_value!(db, CompositeValue, (k, v));
}
