use std::{
    alloc::{Layout, alloc, dealloc},
    ptr::NonNull,
    slice,
    sync::Arc,
};

/// A contiguous memory region holding raw bytes.
///
/// Uses Arc<BufferInner> pattern to support both cheap cloning and automatic cleanup:
/// - Clone just increments Arc's refcount (no data copy)
/// - Drop automatically frees memory from BufferInner when last reference is dropped
/// - Multiple Buffers can safely share the same underlying memory
///
/// Without Arc, we'd face double-free on clone or expensive deep copies.
#[derive(Clone)]
pub struct Buffer {
    inner: Arc<BufferInner>,
    offset: usize,
    length: usize,
}

impl Buffer {
    /// Creates an empty buffer with capacity allocated
    pub fn with_capacity(capacity: usize, alignment: usize) -> Self {
        Self {
            inner: Arc::new(BufferInner::new(capacity, alignment)),
            offset: 0,
            length: 0,
        }
    }

    pub fn slice(&self, offset: usize, length: usize) -> Self {
        assert!(offset + length <= self.length);
        Self {
            inner: self.inner.clone(),
            offset: self.offset + offset,
            length,
        }
    }

    pub fn from_i32_slice(values: &[i32]) -> Self {
        let capacity = values.len() * 4;
        let mut inner = BufferInner::new(capacity, 4);

        // Write the data into the buffer
        unsafe {
            let i32_ptr = inner.ptr.as_ptr() as *mut i32;
            for (i, &value) in values.iter().enumerate() {
                i32_ptr.add(i).write(value);
            }
        }

        // Update length to reflect written data
        inner.length = capacity;

        Self {
            inner: Arc::new(inner),
            offset: 0,
            length: capacity,
        }
    }

    pub fn from_u8_slice(values: &[u8]) -> Self {
        let capacity = values.len();
        let mut inner = BufferInner::new(capacity, 1);

        unsafe {
            let u8_ptr = inner.ptr.as_ptr() as *mut u8;
            for (i, &value) in values.iter().enumerate() {
                u8_ptr.add(i).write(value);
            }
        }

        inner.length = capacity;

        Self {
            inner: Arc::new(inner),
            offset: 0,
            length: capacity,
        }
    }

    /// Read the contents of buffer as a slice of u8
    pub fn as_u8_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr(), self.length) }
    }

    /// Read the contents of buffer as a slice of i32
    pub fn as_i32_slice(&self) -> &[i32] {
        assert_eq!(self.length % 4, 0, "Buffer length not divisible by 4");
        assert_eq!(self.ptr() as usize % 4, 0, "Buffer not aligned for i32");

        unsafe { slice::from_raw_parts(self.ptr() as *const i32, self.length / 4) }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn ptr(&self) -> *const u8 {
        self.inner.offset_ptr(self.offset)
    }
}

/// Inner buffer holding the actual allocated memory.
///
/// We use custom allocation to guarantee proper alignment for typed access.
/// While Vec<u8> may happen to be aligned (allocators often align to 8/16 bytes),
/// the Rust type system only guarantees 1-byte alignment for Vec<u8>.
///
/// To safely reinterpret bytes as i32/i64, we need explicit alignment guarantees.
/// Creating a misaligned reference (&[i32]) is undefined behavior in Rust,
/// even on CPUs that tolerate misaligned access.
struct BufferInner {
    ptr: NonNull<u8>, // Pointer to allocated memory
    length: usize,    // Number of bytes
    capacity: usize,  // Allocated capacity
    layout: Layout,   // For deallocation
}

impl BufferInner {
    fn new(capacity: usize, alignment: usize) -> Self {
        // Allocate memory with correct alignment
        let layout = Layout::from_size_align(capacity, alignment).unwrap();
        let ptr = unsafe { alloc(layout) };

        if ptr.is_null() {
            panic!("Allocation failed!");
        }

        let ptr = NonNull::new(ptr).unwrap();

        Self {
            ptr,
            length: 0,
            capacity,
            layout,
        }
    }

    fn offset_ptr(&self, n: usize) -> *const u8 {
        unsafe { self.ptr.as_ptr().add(n) }
    }

    fn ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for BufferInner {
    fn drop(&mut self) {
        unsafe {
            // Free the allocated memory using the same layout we used to allocate
            dealloc(self.ptr.as_ptr(), self.layout);
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
