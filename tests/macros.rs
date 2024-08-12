/// Create a model of the given key and value type(s).
#[allow(unused)]
macro_rules! create_model {
    (
        $struct_name:ident,
        $key_ty:ty,
        $val_ty:ty,
        [ $( $key_name:ident ), + ],
        [ $( $value_name:ident ), * ]
    ) => {
        #[derive(Model, Debug, PartialEq)]
        #[allow(non_camel_case_types)]
        #[model(impl_from)]
        struct $struct_name {
            $(
                #[entry(position(key))]
                $key_name: $key_ty,
            ) +
            $(
                #[entry(position(value))]
                $value_name: $val_ty,
            ) ?
        }
    };
}

/// Test the given model with the payload data.
#[allow(unused)]
macro_rules! test_model {
    (
        $struct_name:ident,
        [ $( $entry_name:ident: $entry:tt ), + ]
    ) => {
        $(
            let $entry_name = $struct_name::from_values($entry);
        ) +

        let db = Database::builder()
            .create_with_backend(InMemoryBackend::new())
            .unwrap();

        // Insert entries.
        let txn = db.begin_write().unwrap();
        {
            let mut table = txn.open_table($struct_name::DEFINITION).unwrap();
            $(
                let (k, v) = $entry_name.as_values();
                table.insert(k, v).unwrap();
            ) +
        }
        txn.commit().unwrap();

        // Get entries.
        let txn = db.begin_read().unwrap();
        {
            let table = txn.open_table($struct_name::DEFINITION).unwrap();
            // Test `impl_from` and `from_values`.
            $(
                {
                    let (k, _) = $entry_name.as_values();
                    let result = table
                        .get(k)
                        .unwrap()
                        .map(|v| $struct_name::from((k.to_owned(), v.value().to_owned())))
                        .unwrap();

                    assert_eq!(result, $entry_name);
                }
            ) +

            // `impl_from` and `from_guards`.
            let all_entries = table
                .range::<<$struct_name as redb_model::Model<_, _>>::ModelKey>(..)
                .unwrap()
                .map(|result| {
                    result
                        .map(|guards| $struct_name::from_guards((guards.0, guards.1)))
                        .unwrap()
                })
                .collect::<Vec<_>>();

            $(
                assert!(all_entries.contains(&$entry_name));
            ) +
        }
    };
}

/// Create and test models for an integer type.
#[allow(unused)]
macro_rules! test_integer {
    (
        $mod_name:ident,
        $struct_name:ident,
        $ty:ty
    ) => {
        #[cfg(test)]
        mod $mod_name {

            use redb::{backends::InMemoryBackend, Database};
            use redb_model::Model;

            #[test]
            fn test_single() {
                create_model!($struct_name, $ty, $ty, [key_0], [var_0]);
                test_model!($struct_name, [a: (0, 1), b: (2, 3), c: (4, 5)]);
            }

            #[test]
            fn test_composite() {
                create_model!($struct_name, $ty, $ty, [key_0, key_1, key_2], [var_0, var_1, var_2]);
                test_model!($struct_name, [a: ((0, 1, 2), (3, 4, 5)), b: ((6, 7, 8), (9, 10, 11)), c: ((12, 13, 14), (15, 16, 17))]);
            }
        }
    };
}

/// Create and test models for a float type.
#[allow(unused)]
macro_rules! test_float {
    (
        $mod_name:ident,
        $struct_name:ident,
        $ty:ty
    ) => {
        #[cfg(test)]
        mod $mod_name {

            use redb::{backends::InMemoryBackend, Database};
            use redb_model::Model;

            #[test]
            fn test_single() {
                create_model!($struct_name, u8, $ty, [key_0], [var_0]);
                test_model!($struct_name, [a: (0, 1.0), b: (2, 3.0), c: (4, 5.0)]);
            }

            #[test]
            fn test_composite() {
                create_model!($struct_name, u8, $ty, [key_0, key_1, key_2], [var_0, var_1, var_2]);
                test_model!($struct_name, [a: ((0, 1, 2), (3.0, 4.0, 5.0)), b: ((6, 7, 8), (9.0, 10.0, 11.0)), c: ((12, 13, 14), (15.0, 16.0, 17.0))]);
            }
        }
    };
}
