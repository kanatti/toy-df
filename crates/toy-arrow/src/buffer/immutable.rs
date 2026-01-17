use std::{alloc::Layout, ptr::NonNull, slice, sync::Arc};

use crate::{bytes::Bytes, native::NativeType};

/// An immutable, reference-counted buffer of raw bytes with support for zero-copy slicing.
///
/// ## Design
///
/// ```text
/// Buffer A                    Buffer B (slice of A)
/// ┌──────────────────┐        ┌──────────────────┐
/// │ bytes: Arc ──────┼───┐    │ bytes: Arc ──────┼───┐
/// │ offset: 0        │   │    │ offset: 4        │   │
/// │ length: 12       │   │    │ length: 4        │   │
/// └──────────────────┘   │    └──────────────────┘   │
///                        │                           │
///                        ▼                           │
///                       Bytes ◄──────────────────────┘
///                   ┌─────────────────────────────┐
///                   │ ptr: ───► [bytes...12 total]│
///                   │ layout: (size=12, align=4)  │
///                   └─────────────────────────────┘
/// ```
///
/// ## Why Arc<Bytes>?
///
/// - **Cheap clone**: Just increments refcount, no data copy
/// - **Safe sharing**: Multiple Buffers can reference the same memory
/// - **Automatic cleanup**: Memory freed when last reference drops
/// - **Zero-copy slicing**: Slices share the same Bytes
///
/// Without Arc, we'd face double-free on clone or need expensive deep copies.
#[derive(Clone)]
pub struct Buffer {
    bytes: Arc<Bytes>,
    /// Byte offset into Bytes's allocation. Enables zero-copy slicing:
    /// a slice is just a new Buffer with adjusted offset/length, sharing the same Arc.
    offset: usize,
    /// Number of bytes visible through this Buffer (may be less than Bytes's capacity).
    length: usize,
}

impl Buffer {
    /// Creates an empty buffer with pre-allocated capacity.
    ///
    /// Useful when you know the final size upfront and want to avoid reallocations.
    /// The buffer starts with length 0 (no visible data) but has memory reserved.
    pub fn with_capacity(capacity: usize, alignment: usize) -> Self {
        Self {
            bytes: Arc::new(Bytes::new(capacity, alignment)),
            offset: 0,
            length: 0,
        }
    }

    /// Creates a zero-copy slice of this buffer.
    ///
    /// The returned Buffer shares the same underlying memory (via Arc clone).
    /// No bytes are copied - only the offset and length change.
    ///
    /// ## Panics
    /// Panics if `offset + length` exceeds this buffer's length.
    pub fn slice(&self, offset: usize, length: usize) -> Self {
        assert!(offset + length <= self.length);
        Self {
            bytes: self.bytes.clone(), // Arc clone = refcount increment, not data copy
            offset: self.offset + offset,
            length,
        }
    }

    /// Creates a buffer from an i32 slice, copying the data.
    ///
    /// Allocates with 4-byte alignment to allow safe reinterpretation as &[i32].
    pub fn from_i32_slice(values: &[i32]) -> Self {
        let capacity = values.len() * 4;
        let mut bytes = Bytes::new(capacity, 4);

        unsafe {
            let i32_ptr = bytes.ptr().as_ptr() as *mut i32;
            for (i, &value) in values.iter().enumerate() {
                i32_ptr.add(i).write(value);
            }
        }

        bytes.set_length(capacity);

        Self {
            bytes: Arc::new(bytes),
            offset: 0,
            length: capacity,
        }
    }

    /// Creates a buffer from a u8 slice, copying the data.
    ///
    /// Uses 1-byte alignment (no alignment requirement for u8).
    pub fn from_u8_slice(values: &[u8]) -> Self {
        let capacity = values.len();
        let mut bytes = Bytes::new(capacity, 1);

        unsafe {
            let u8_ptr = bytes.ptr().as_ptr() as *mut u8;
            for (i, &value) in values.iter().enumerate() {
                u8_ptr.add(i).write(value);
            }
        }

        bytes.set_length(capacity);

        Self {
            bytes: Arc::new(bytes),
            offset: 0,
            length: capacity,
        }
    }

    /// Returns the buffer contents as a byte slice.
    pub fn as_u8_slice(&self) -> &[u8] {
        // SAFETY: ptr() returns valid pointer, length is tracked correctly
        unsafe { slice::from_raw_parts(self.ptr(), self.length) }
    }

    /// Returns the buffer contents as an i32 slice.
    ///
    /// ## Panics
    /// - If length is not divisible by 4 (size of i32)
    /// - If pointer is not 4-byte aligned
    ///
    /// Prefer using `ScalarBuffer<i32>` which enforces these at construction time.
    pub fn as_i32_slice(&self) -> &[i32] {
        assert_eq!(self.length % 4, 0, "Buffer length not divisible by 4");
        assert_eq!(self.ptr() as usize % 4, 0, "Buffer not aligned for i32");

        unsafe { slice::from_raw_parts(self.ptr() as *const i32, self.length / 4) }
    }

    /// Returns the number of bytes in this buffer.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns a raw pointer to the start of this buffer's data.
    ///
    /// The pointer accounts for any slice offset, so it points to the first
    /// byte visible through this Buffer, not necessarily the start of the allocation.
    pub fn ptr(&self) -> *const u8 {
        self.bytes.offset_ptr(self.offset)
    }
}

impl<T: NativeType> From<Vec<T>> for Buffer {
    /// Takes ownership of a Vec's allocation without copying.
    ///
    /// ## How it works
    /// 1. Extract the Vec's pointer, length, and capacity
    /// 2. Create a Layout matching the Vec's allocation
    /// 3. `mem::forget(value)` prevents Vec's destructor from freeing the memory
    /// 4. Bytes now owns the memory and will free it on drop
    ///
    /// This is safe because:
    /// - Vec guarantees proper alignment for T
    /// - We preserve the exact Layout for correct deallocation
    /// - The Vec is forgotten, so no double-free
    fn from(value: Vec<T>) -> Self {
        let ptr = NonNull::new(value.as_ptr() as _).unwrap();
        let length = value.len() * T::get_byte_width();
        // Use capacity (not len) to get the actual allocation size
        let layout = Layout::array::<T>(value.capacity()).unwrap();
        let bytes = Arc::new(Bytes::from_raw_parts(ptr, length, layout));
        // CRITICAL: forget the Vec so it doesn't free the memory we just took ownership of
        std::mem::forget(value);
        Self {
            bytes,
            offset: 0,
            length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_i32_slice() {
        let values = vec![1i32, 2, 3, 4, 5];
        let buffer = Buffer::from_i32_slice(&values);

        assert_eq!(buffer.len(), 20); // 5 * 4 bytes
        assert_eq!(buffer.as_i32_slice(), &values[..]);
    }

    #[test]
    fn test_from_u8_slice() {
        let values = vec![0u8, 1, 2, 3, 255];
        let buffer = Buffer::from_u8_slice(&values);

        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.as_u8_slice(), &values[..]);
    }

    #[test]
    fn test_buffer_alignment_i32() {
        let values = vec![100i32, 200, 300];
        let buffer = Buffer::from_i32_slice(&values);

        let ptr = buffer.as_u8_slice().as_ptr() as usize;
        assert_eq!(ptr % 4, 0, "Buffer should be 4-byte aligned for i32");
    }

    #[test]
    fn test_buffer_clone_shares_memory() {
        let values = vec![42i32, 84, 126];
        let buffer1 = Buffer::from_i32_slice(&values);
        let buffer2 = buffer1.clone();

        // Same data
        assert_eq!(buffer1.as_i32_slice(), buffer2.as_i32_slice());

        // Same pointer (shared memory)
        assert_eq!(
            buffer1.as_u8_slice().as_ptr(),
            buffer2.as_u8_slice().as_ptr()
        );
    }

    #[test]
    fn test_buffer_empty() {
        let buffer = Buffer::with_capacity(10, 1);
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.as_u8_slice().len(), 0);
    }

    #[test]
    fn test_i32_slice_misaligned_panics() {
        // Create a u8 buffer with odd length (not divisible by 4)
        let values = vec![1u8, 2, 3];
        let buffer = Buffer::from_u8_slice(&values);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = buffer.as_i32_slice();
        }));

        assert!(result.is_err(), "Should panic on misaligned i32 access");
    }

    #[test]
    fn test_slice_basic() {
        let values = vec![0u8, 1, 2, 3, 4, 5, 6, 7];
        let buffer = Buffer::from_u8_slice(&values);

        let sliced = buffer.slice(2, 4);

        assert_eq!(sliced.len(), 4);
        assert_eq!(sliced.as_u8_slice(), &[2, 3, 4, 5]);
    }

    #[test]
    fn test_slice_shares_memory() {
        let values = vec![0u8, 1, 2, 3, 4, 5, 6, 7];
        let buffer = Buffer::from_u8_slice(&values);
        let sliced = buffer.slice(2, 4);

        // Sliced pointer should be original pointer + offset
        let original_ptr = buffer.as_u8_slice().as_ptr();
        let sliced_ptr = sliced.as_u8_slice().as_ptr();
        assert_eq!(sliced_ptr, unsafe { original_ptr.add(2) });
    }

    #[test]
    fn test_slice_of_slice() {
        let values = vec![0u8, 1, 2, 3, 4, 5, 6, 7];
        let buffer = Buffer::from_u8_slice(&values);

        let slice1 = buffer.slice(2, 5); // [2, 3, 4, 5, 6]
        let slice2 = slice1.slice(1, 3); // [3, 4, 5]

        assert_eq!(slice2.len(), 3);
        assert_eq!(slice2.as_u8_slice(), &[3, 4, 5]);
    }

    #[test]
    fn test_slice_i32() {
        let values = vec![10i32, 20, 30, 40, 50];
        let buffer = Buffer::from_i32_slice(&values);

        // Slice 2 i32s starting at the second element (offset 4 bytes, length 8 bytes)
        let sliced = buffer.slice(4, 8);

        assert_eq!(sliced.as_i32_slice(), &[20, 30]);
    }

    #[test]
    #[should_panic]
    fn test_slice_out_of_bounds() {
        let values = vec![0u8, 1, 2, 3, 4];
        let buffer = Buffer::from_u8_slice(&values);

        // This should panic: offset 3 + length 4 = 7 > 5
        let _ = buffer.slice(3, 4);
    }
}
