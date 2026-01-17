use std::cmp::max;

use crate::bytes::Bytes;

pub struct MutableBuffer {
    bytes: Bytes,
}

impl MutableBuffer {
    pub fn with_capacity(capacity: usize, alignment: usize) -> Self {
        let bytes = Bytes::new(capacity, alignment);
        Self { bytes }
    }

    pub fn push(&mut self, value: u8) {
        // 1. Calculate the space needed
        // 2. If not then grow the buffer
        // 3. Write
        let new_length = self.bytes.length() + 1;
        if new_length > self.bytes.capacity() {
            let new_capacity = max(new_length, self.bytes.capacity() * 2);
            self.grow(new_capacity);
        }
        let ptr = self.bytes.offset_ptr_mut(self.bytes.length());
        unsafe {
            ptr.write(value);
        }
        self.bytes.set_length(new_length);
    }

    pub fn extend_from_slice(&mut self, values: &[u8]) {
        // 1. Calculate the space needed
        // 2. If not then grow the buffer
        // 3. Write
        let new_length = self.bytes.length() + values.len();
        if new_length > self.bytes.capacity() {
            let new_capacity = max(new_length, self.bytes.capacity() * 2);
            self.grow(new_capacity);
        }

        let src_ptr = values.as_ptr();
        let dst_ptr = self.bytes.offset_ptr_mut(self.bytes.length());
        unsafe {
            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, values.len());
        }
        self.bytes.set_length(new_length);
    }

    fn grow(&mut self, capacity: usize) {
        let mut bytes = Bytes::new(capacity, self.bytes.alignment());
        let src_ptr = self.bytes.ptr().as_ptr() as *const u8;
        let dst_ptr = bytes.ptr().as_ptr();

        unsafe {
            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, self.bytes.length());
        }

        bytes.set_length(self.bytes.length());
        self.bytes = bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_capacity() {
        let buffer = MutableBuffer::with_capacity(10, 1);
        assert_eq!(buffer.bytes.length(), 0);
        assert_eq!(buffer.bytes.capacity(), 10);
    }

    #[test]
    fn test_push() {
        let mut buffer = MutableBuffer::with_capacity(5, 1);
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        assert_eq!(buffer.bytes.length(), 3);

        // Verify data
        unsafe {
            let ptr = buffer.bytes.ptr().as_ptr();
            assert_eq!(*ptr, 1);
            assert_eq!(*ptr.add(1), 2);
            assert_eq!(*ptr.add(2), 3);
        }
    }

    #[test]
    fn test_extend_from_slice() {
        let mut buffer = MutableBuffer::with_capacity(10, 1);
        buffer.extend_from_slice(&[1, 2, 3, 4, 5]);

        assert_eq!(buffer.bytes.length(), 5);

        // Verify data
        unsafe {
            let ptr = buffer.bytes.ptr().as_ptr();
            for i in 0..5 {
                assert_eq!(*ptr.add(i), (i + 1) as u8);
            }
        }
    }

    #[test]
    fn test_push_grows() {
        let mut buffer = MutableBuffer::with_capacity(2, 1);
        buffer.push(1);
        buffer.push(2);

        assert_eq!(buffer.bytes.capacity(), 2);

        // This should trigger growth
        buffer.push(3);

        assert_eq!(buffer.bytes.length(), 3);
        assert!(buffer.bytes.capacity() >= 3);

        // Verify all data preserved
        unsafe {
            let ptr = buffer.bytes.ptr().as_ptr();
            assert_eq!(*ptr, 1);
            assert_eq!(*ptr.add(1), 2);
            assert_eq!(*ptr.add(2), 3);
        }
    }

    #[test]
    fn test_extend_grows() {
        let mut buffer = MutableBuffer::with_capacity(5, 1);
        buffer.extend_from_slice(&[1, 2, 3]);

        // This should trigger growth
        buffer.extend_from_slice(&[4, 5, 6, 7, 8]);

        assert_eq!(buffer.bytes.length(), 8);
        assert!(buffer.bytes.capacity() >= 8);

        // Verify all data
        unsafe {
            let ptr = buffer.bytes.ptr().as_ptr();
            for i in 0..8 {
                assert_eq!(*ptr.add(i), (i + 1) as u8);
            }
        }
    }
}
