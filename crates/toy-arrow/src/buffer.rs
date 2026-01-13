use std::{
    alloc::{Layout, alloc, dealloc},
    ptr::NonNull,
    slice,
    sync::Arc,
};

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
    len: usize,       // Number of bytes
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
            len: 0,
            capacity,
            layout,
        }
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
}

impl Buffer {
    pub fn with_capacity(capacity: usize, alignment: usize) -> Self {
        Self {
            inner: Arc::new(BufferInner::new(capacity, alignment)),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.inner.ptr.as_ptr(), self.inner.len) }
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
        inner.len = capacity;

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn as_i32_slice(&self) -> &[i32] {
        assert_eq!(self.inner.len % 4, 0, "Buffer length not divisible by 4");
        assert_eq!(
            self.inner.ptr.as_ptr() as usize % 4,
            0,
            "Buffer not aligned for i32"
        );

        unsafe { slice::from_raw_parts(self.inner.ptr.as_ptr() as *const i32, self.inner.len / 4) }
    }

    pub fn len(&self) -> usize {
        self.inner.len
    }
}
