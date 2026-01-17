use std::{
    alloc::{Layout, alloc, dealloc},
    cmp::max,
    mem,
    ptr::NonNull,
};

use crate::{Buffer, buffer::Bytes, native::NativeType};

/// A mutable, growable buffer for building Arrow buffers.
///
/// ## Design
///
/// `MutableBuffer` is the construction phase of the immutable `Buffer`. It:
/// - Owns a raw memory allocation exclusively (no Arc sharing)
/// - Grows dynamically using a 2x strategy when capacity is exceeded
/// - Converts to immutable `Buffer` via `into_buffer()` (typestate pattern)
///
/// ## Why separate from Buffer?
///
/// - **Clear intent**: Mutable construction vs immutable use
/// - **Type safety**: Can't mutate after freezing (enforced by move semantics)
/// - **No overhead**: Immutable buffers don't pay for growth logic
pub struct MutableBuffer {
    ptr: NonNull<u8>,
    len: usize,
    layout: Layout,
}

impl MutableBuffer {
    /// Creates a new `MutableBuffer` with the specified capacity and alignment.
    ///
    /// The buffer starts empty (length 0) but has memory pre-allocated.
    ///
    /// ## Arguments
    ///
    /// * `capacity` - Number of bytes to allocate
    /// * `alignment` - Memory alignment in bytes (typically 8, 16, or 64 for SIMD)
    ///
    /// ## Panics
    ///
    /// Panics if allocation fails or alignment is invalid.
    pub fn with_capacity(capacity: usize, alignment: usize) -> Self {
        let layout = Layout::from_size_align(capacity, alignment).unwrap();
        let ptr = unsafe { alloc(layout) };

        if ptr.is_null() {
            panic!("Allocation failed!");
        }

        let ptr = NonNull::new(ptr).unwrap();

        Self {
            ptr,
            len: 0,
            layout,
        }
    }

    /// Appends a single byte to the buffer.
    ///
    /// Automatically grows the buffer if needed using a 2x growth strategy.
    pub fn push(&mut self, value: u8) {
        let additional = 1;
        self.reserve(additional);

        unsafe {
            let ptr = self.ptr.as_ptr().add(self.len);
            ptr.write(value);
        }

        self.len += additional;
    }

    /// Appends a slice of typed values to the buffer.
    ///
    /// The values are reinterpreted as bytes and copied into the buffer.
    /// Automatically grows the buffer if needed.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let mut buf = MutableBuffer::with_capacity(16, 4);
    /// buf.extend_from_slice(&[1i32, 2, 3]);  // Appends 12 bytes
    /// buf.extend_from_slice(&[10u8, 20, 30]); // Appends 3 bytes
    /// ```
    pub fn extend_from_slice<T: NativeType>(&mut self, values: &[T]) {
        let additional_bytes = values.len() * T::get_byte_width();
        self.reserve(additional_bytes);

        let src_ptr = values.as_ptr() as *const u8;
        unsafe {
            let dst_ptr = self.ptr.as_ptr().add(self.len);
            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, additional_bytes);
        }
        self.len += additional_bytes;
    }

    /// Ensures there is capacity for at least `additional` more bytes.
    ///
    /// If current capacity is sufficient, does nothing. Otherwise, reallocates
    /// with growth strategy: `max(current * 2, required)`.
    ///
    /// This allows pre-allocating when you know the size upfront, avoiding
    /// multiple reallocations.
    pub fn reserve(&mut self, additional: usize) {
        let required_capacity = self.len + additional;
        if required_capacity > self.layout.size() {
            let new_capacity = max(required_capacity, self.layout.size() * 2);
            self.grow(new_capacity);
        }
    }

    /// Converts this `MutableBuffer` into an immutable `Buffer`.
    ///
    /// This consumes the `MutableBuffer` and transfers ownership of the memory
    /// to the returned `Buffer`. The memory is not copied - just wrapped in
    /// the immutable `Buffer` type.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let mut buf = MutableBuffer::with_capacity(10, 8);
    /// buf.extend_from_slice(&[1u8, 2, 3]);
    /// let buffer = buf.into_buffer();
    /// // buf is now consumed, can't use it anymore
    /// ```
    pub fn into_buffer(self) -> Buffer {
        let bytes = unsafe { Bytes::new(self.ptr, self.len, self.layout) };
        mem::forget(self); // Prevent drop from running, we have transferred memory to bytes
        bytes.into()
    }

    /// Returns the number of bytes currently in the buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the total capacity in bytes.
    pub fn capacity(&self) -> usize {
        self.layout.size()
    }

    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Reallocates the buffer to a new capacity.
    ///
    /// Copies existing data to the new allocation and frees the old one.
    fn grow(&mut self, capacity: usize) {
        let new_layout = Layout::from_size_align(capacity, self.layout.align()).unwrap();
        let new_ptr = unsafe { alloc(new_layout) };

        if new_ptr.is_null() {
            panic!("Allocation failed!");
        }

        let new_ptr = NonNull::new(new_ptr).unwrap();

        // Copy old data to new allocation
        unsafe {
            std::ptr::copy_nonoverlapping(self.ptr.as_ptr(), new_ptr.as_ptr(), self.len);
        }

        // Deallocate old memory
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }

        self.ptr = new_ptr;
        self.layout = new_layout;
    }
}

impl Drop for MutableBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_capacity() {
        let buffer = MutableBuffer::with_capacity(10, 1);
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 10);
    }

    #[test]
    fn test_push() {
        let mut buffer = MutableBuffer::with_capacity(5, 1);
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        assert_eq!(buffer.len(), 3);

        // Verify data
        unsafe {
            let ptr = buffer.ptr.as_ptr();
            assert_eq!(*ptr, 1);
            assert_eq!(*ptr.add(1), 2);
            assert_eq!(*ptr.add(2), 3);
        }
    }

    #[test]
    fn test_extend_from_slice() {
        let mut buffer = MutableBuffer::with_capacity(10, 1);
        buffer.extend_from_slice(&[1u8, 2, 3, 4, 5]);

        assert_eq!(buffer.len(), 5);

        // Verify data
        unsafe {
            let ptr = buffer.ptr.as_ptr();
            for i in 0..5 {
                assert_eq!(*ptr.add(i), (i + 1) as u8);
            }
        }
    }

    #[test]
    fn test_extend_i32_slice() {
        let mut buffer = MutableBuffer::with_capacity(16, 4);
        buffer.extend_from_slice(&[1i32, 2, 3]);

        assert_eq!(buffer.len(), 12); // 3 * 4 bytes

        // Verify data as i32
        unsafe {
            let ptr = buffer.ptr.as_ptr() as *const i32;
            assert_eq!(*ptr, 1);
            assert_eq!(*ptr.add(1), 2);
            assert_eq!(*ptr.add(2), 3);
        }
    }

    #[test]
    fn test_push_grows() {
        let mut buffer = MutableBuffer::with_capacity(2, 1);
        buffer.push(1);
        buffer.push(2);

        assert_eq!(buffer.capacity(), 2);

        // This should trigger growth
        buffer.push(3);

        assert_eq!(buffer.len(), 3);
        assert!(buffer.capacity() >= 3);

        // Verify all data preserved
        unsafe {
            let ptr = buffer.ptr.as_ptr();
            assert_eq!(*ptr, 1);
            assert_eq!(*ptr.add(1), 2);
            assert_eq!(*ptr.add(2), 3);
        }
    }

    #[test]
    fn test_extend_grows() {
        let mut buffer = MutableBuffer::with_capacity(5, 1);
        buffer.extend_from_slice(&[1u8, 2, 3]);

        // This should trigger growth
        buffer.extend_from_slice(&[4u8, 5, 6, 7, 8]);

        assert_eq!(buffer.len(), 8);
        assert!(buffer.capacity() >= 8);

        // Verify all data
        unsafe {
            let ptr = buffer.ptr.as_ptr();
            for i in 0..8 {
                assert_eq!(*ptr.add(i), (i + 1) as u8);
            }
        }
    }

    #[test]
    fn test_into_buffer() {
        let mut mutable = MutableBuffer::with_capacity(10, 8);
        mutable.extend_from_slice(&[1u8, 2, 3, 4, 5]);

        let buffer = mutable.into_buffer();

        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.as_u8_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_reserve() {
        let mut buffer = MutableBuffer::with_capacity(5, 1);
        buffer.extend_from_slice(&[1u8, 2, 3]);

        assert_eq!(buffer.len(), 3);

        // Reserve space for 10 more bytes
        buffer.reserve(10);

        assert!(buffer.capacity() >= buffer.len() + 10); // At least len + additional
        assert_eq!(buffer.len(), 3); // Length unchanged

        // Verify existing data preserved
        unsafe {
            let ptr = buffer.ptr.as_ptr();
            assert_eq!(*ptr, 1);
            assert_eq!(*ptr.add(1), 2);
            assert_eq!(*ptr.add(2), 3);
        }
    }
}
