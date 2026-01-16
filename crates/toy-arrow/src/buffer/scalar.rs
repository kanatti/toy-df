use std::{marker::PhantomData, ops::Deref};

use crate::{Buffer, native::NativeType};

/// A type-safe view over a [`Buffer`] that interprets raw bytes as values of type `T`.
///
/// ## Why ScalarBuffer?
///
/// [`Buffer`] stores raw bytes with no type information. To work with typed data (i32, f64, etc.),
/// you need to interpret those bytes correctly. ScalarBuffer provides:
///
/// 1. **Type safety**: Can't accidentally read i32 bytes as f64
/// 2. **Alignment enforcement**: Validates buffer is properly aligned for type T at construction
/// 3. **Ergonomic access**: Derefs to `&[T]` so you get slice methods for free
///
/// ## Example
/// ```text
/// Buffer (raw bytes):     [0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00]
/// ScalarBuffer<i32>:      [1, 2]  (interprets every 4 bytes as an i32)
/// ```
///
/// ## Zero-Copy
/// Like Buffer, cloning a ScalarBuffer is cheap (Arc refcount increment).
/// Slicing creates a new view without copying data.
#[derive(Clone)]
pub struct ScalarBuffer<T: NativeType> {
    buffer: Buffer,
    /// PhantomData tells the compiler this struct is "associated with" type T,
    /// even though it doesn't store any T values directly. This enables:
    /// - Correct variance (ScalarBuffer<i32> is not ScalarBuffer<i64>)
    /// - Drop check (though T: Copy so not relevant here)
    _phantom: PhantomData<T>,
}

impl<T: NativeType> ScalarBuffer<T> {
    /// Returns a zero-copy slice of this buffer.
    ///
    /// Parameters are in units of T (not bytes):
    /// - `offset`: number of T elements to skip
    /// - `len`: number of T elements in the slice
    ///
    /// Internally converts to byte offsets for the underlying Buffer.
    pub fn slice(&self, offset: usize, len: usize) -> Self {
        let byte_offset = offset * T::get_byte_width();
        let byte_len = len * T::get_byte_width();
        self.buffer.slice(byte_offset, byte_len).into()
    }
}

impl<T: NativeType> From<Buffer> for ScalarBuffer<T> {
    /// Wraps a Buffer as a ScalarBuffer<T>.
    ///
    /// ## Panics
    /// Panics if the buffer's pointer is not aligned for type T.
    ///
    /// ## Why alignment matters
    /// Creating a reference to misaligned data is undefined behavior in Rust,
    /// even on CPUs that support unaligned access. For example, an i32 requires
    /// 4-byte alignment. If we allowed a buffer at address 0x1001 to become
    /// ScalarBuffer<i32>, the Deref impl would create an invalid &[i32].
    fn from(buffer: Buffer) -> ScalarBuffer<T> {
        assert!(
            buffer.ptr() as usize % T::get_alignment() == 0,
            "Buffer pointer not aligned: address {:x} needs {}-byte alignment",
            buffer.ptr() as usize,
            T::get_alignment()
        );
        ScalarBuffer {
            buffer,
            _phantom: PhantomData,
        }
    }
}

impl<T: NativeType> From<Vec<T>> for ScalarBuffer<T> {
    /// Creates a ScalarBuffer from a Vec, taking ownership of the Vec's allocation.
    ///
    /// This is zero-copy: the Vec's memory is transferred directly to the Buffer.
    /// The Vec is consumed (forgotten) and its memory will be freed when the
    /// ScalarBuffer is dropped.
    fn from(values: Vec<T>) -> ScalarBuffer<T> {
        Buffer::from(values).into()
    }
}

impl<T: NativeType> Deref for ScalarBuffer<T> {
    type Target = [T];

    /// Provides direct slice access to the buffer's contents.
    ///
    /// ## Safety
    /// This is safe because:
    /// 1. Alignment is verified at construction (From<Buffer> checks this)
    /// 2. NativeType is a sealed trait only implemented for valid primitive types
    /// 3. The buffer's lifetime is tied to self, so the slice can't outlive the data
    /// 4. Buffer length is always a multiple of T's size (enforced by construction)
    fn deref(&self) -> &Self::Target {
        // SAFETY: See doc comment above. The key invariants are:
        // - Pointer is aligned (checked in From<Buffer>)
        // - Length is valid (buffer.len() / byte_width gives correct count)
        // - Memory is valid for reads (owned by BufferInner via Arc)
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

    #[test]
    fn test_from_vec() {
        let scalar: ScalarBuffer<i32> = vec![1, 2, 3].into();
        assert_eq!(&*scalar, &[1, 2, 3]);
        assert_eq!(scalar.len(), 3);

        let scalar: ScalarBuffer<f64> = vec![1.5, 2.5].into();
        assert_eq!(&*scalar, &[1.5, 2.5]);
    }

    #[test]
    fn test_slice() {
        let scalar: ScalarBuffer<i32> = vec![10, 20, 30, 40, 50].into();

        let sliced = scalar.slice(1, 3);
        assert_eq!(&*sliced, &[20, 30, 40]);

        // slice of slice
        let sliced2 = sliced.slice(1, 1);
        assert_eq!(&*sliced2, &[30]);
    }
}
