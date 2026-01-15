use std::{marker::PhantomData, ops::Deref};

use crate::{Buffer, native::NativeType};

/// Buffer stores physical bytes. ScalarBuffer<T> wraps it to provide
/// typed access to logical values (i32, f64, etc.) with compile-time safety.
pub struct ScalarBuffer<T: NativeType> {
    buffer: Buffer,
    _phantom: PhantomData<T>,
}

impl<T: NativeType> From<Buffer> for ScalarBuffer<T> {
    fn from(buffer: Buffer) -> ScalarBuffer<T> {
        assert!(
            buffer.ptr() as usize % T::get_alignment() == 0,
            "Buffer pointer not aligned: address {:x} needs {}-byte alignment",
            buffer.ptr() as usize,
            T::get_alignment()
        );
        ScalarBuffer {
            buffer: buffer,
            _phantom: PhantomData,
        }
    }
}

impl<T: NativeType> Deref for ScalarBuffer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(
                self.buffer.ptr() as *const T,
                self.buffer.len() / T::get_byte_width(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_i32_buffer() {
        let buffer = Buffer::from_i32_slice(&[1, 2, 3, 4, 5]);
        let scalar: ScalarBuffer<i32> = buffer.into();

        assert_eq!(scalar.len(), 5);
        assert_eq!(scalar[0], 1);
        assert_eq!(scalar[4], 5);
        assert_eq!(&*scalar, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_from_u8_buffer() {
        let buffer = Buffer::from_u8_slice(&[10, 20, 30]);
        let scalar: ScalarBuffer<u8> = buffer.into();

        assert_eq!(scalar.len(), 3);
        assert_eq!(&*scalar, &[10, 20, 30]);
    }

    #[test]
    fn test_deref_coercion() {
        let buffer = Buffer::from_i32_slice(&[100, 200, 300]);
        let scalar: ScalarBuffer<i32> = buffer.into();

        let slice: &[i32] = &scalar;
        assert_eq!(slice, &[100, 200, 300]);
        assert_eq!(scalar.first(), Some(&100));
        assert_eq!(scalar.iter().sum::<i32>(), 600);
    }

    #[test]
    #[should_panic(expected = "Buffer pointer not aligned")]
    fn test_unaligned_panics() {
        let buffer = Buffer::from_u8_slice(&[0, 1, 2, 3, 4, 5, 6, 7]);
        let sliced = buffer.slice(1, 4); // offset 1 = misaligned for i32
        let _: ScalarBuffer<i32> = sliced.into();
    }
}
