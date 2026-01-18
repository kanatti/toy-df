use crate::{native::NativeType, schema::DataType};

pub trait PrimitiveType: 'static {
    type Native: NativeType;
    const DATA_TYPE: DataType;
}

macro_rules! make_primitive_type {
    ($name:ident, $native_ty:ty, $data_ty:expr) => {
        pub struct $name;

        impl PrimitiveType for $name {
            type Native = $native_ty;
            const DATA_TYPE: DataType = $data_ty;
        }
    };
}

make_primitive_type!(Int8Type, i8, DataType::Int8);
make_primitive_type!(Int16Type, i16, DataType::Int16);
make_primitive_type!(Int32Type, i32, DataType::Int32);
make_primitive_type!(Int64Type, i64, DataType::Int64);
make_primitive_type!(UInt8Type, u8, DataType::UInt8);
make_primitive_type!(UInt16Type, u16, DataType::UInt16);
make_primitive_type!(UInt32Type, u32, DataType::UInt32);
make_primitive_type!(UInt64Type, u64, DataType::UInt64);
make_primitive_type!(Float32Type, f32, DataType::Float32);
make_primitive_type!(Float64Type, f64, DataType::Float64);
