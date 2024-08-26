//! Type tests.
#[macro_use]
mod macros;

test_integer!(i8, i8Model, i8);
test_integer!(i16, i16Model, i16);
test_integer!(i32, i32Model, i32);
test_integer!(i64, i64Model, i64);
test_integer!(i128, i128Model, i128);

test_integer!(u8, u8Model, u8);
test_integer!(u16, u16Model, u16);
test_integer!(u32, u32Model, u32);
test_integer!(u64, u64Model, u64);
test_integer!(u128, u128Model, u128);

test_float!(f32, f32Model, f32);
test_float!(f64, f32Model, f64);
