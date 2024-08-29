/// A typed single value.
/// Larger variants are boxed to keep enum size smaller.
pub enum ScalarValue {
    Boolean(Option<bool>),
    Int8(Option<i8>),
    Int16(Option<i16>),
    Int32(Option<i32>),
    Int64(Option<i64>),
    UInt8(Option<u8>),
    UInt16(Option<u16>),
    UInt32(Option<u32>),
    UInt64(Option<Box<u64>>),
    Float32(Option<f32>),
    Float64(Option<Box<f64>>),
    String(Option<Box<String>>),
    Binary(Option<Box<Vec<u8>>>),
}

// Macro to implement Into<ScalarValue> for simple types.
macro_rules! impl_into_scalar {
    ($t:ty, $v:ident) => {
        impl Into<ScalarValue> for $t {
            fn into(self) -> ScalarValue {
                ScalarValue::$v(Some(self))
            }
        }
    };
}

macro_rules! impl_into_scalar_boxed {
    ($t:ty, $v:ident) => {
        impl Into<ScalarValue> for $t {
            fn into(self) -> ScalarValue {
                ScalarValue::$v(Some(Box::new(self)))
            }
        }
    };
}

impl_into_scalar!(bool, Boolean);
impl_into_scalar!(i8, Int8);
impl_into_scalar!(i16, Int16);
impl_into_scalar!(i32, Int32);
impl_into_scalar!(i64, Int64);
impl_into_scalar!(u8, UInt8);
impl_into_scalar!(u16, UInt16);
impl_into_scalar!(u32, UInt32);
impl_into_scalar_boxed!(u64, UInt64);
impl_into_scalar!(f32, Float32);
impl_into_scalar_boxed!(f64, Float64);
impl_into_scalar_boxed!(String, String);
impl_into_scalar_boxed!(Vec<u8>, Binary);
