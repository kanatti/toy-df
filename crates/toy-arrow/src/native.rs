// Trick to seal a public trait
mod private {
    pub trait Sealed {}
}

/// Rust types that can be stored in Arrow buffers as raw bytes
///
/// This trait enables generic handling of primitive types (i32, f64, etc.)
/// through `ScalarBuffer<T>` instead of separate methods like `as_i32_slice()`,
/// `as_f64_slice()` for each type.
pub trait NativeType: private::Sealed + Copy + Send + Sync + 'static {
    fn get_byte_width() -> usize {
        std::mem::size_of::<Self>()
    }

    fn get_alignment() -> usize {
        std::mem::align_of::<Self>()
    }
}

macro_rules! impl_native_type {
    ($($t:ty),*) => {
        $(
            impl private::Sealed for $t {}
            impl NativeType for $t {}
        )*
    };
}

impl_native_type!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);
