use crate::Buffer;

pub struct NullBuffer {
    buffer: Buffer,    // storage
    len: usize,        // length of bits
    null_count: usize, // count of nulls
}

impl NullBuffer {
    pub fn new(buffer: Buffer, len: usize) -> Self {
        let mut null_count = 0;
        for i in 0..len {
            if is_null_bit(&buffer, i) {
                null_count += 1;
            }
        }

        Self {
            buffer,
            len,
            null_count,
        }
    }

    pub fn from_bools(bools: &[bool]) -> Self {
        let buffer = Self::pack_bools(bools);
        Self::new(buffer, bools.len())
    }

    pub fn null_count(&self) -> usize {
        self.null_count
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_null(&self, idx: usize) -> bool {
        assert!(idx < self.len, "index out of bounds");
        is_null_bit(&self.buffer, idx)
    }

    fn pack_bools(bools: &[bool]) -> Buffer {
        let num_bytes = (bools.len() + 7) / 8;
        // TODO: vec can be avoided.
        let mut bytes = vec![0u8; num_bytes];

        for (i, &bit) in bools.iter().enumerate() {
            if bit {
                // i / 8 -> which bytes contain bit i
                // i % 8 -> position of bit in that byte
                // 1 << (i % 8) -> Creates a mask with just i set
                // |= -> bitwise OR
                bytes[i / 8] |= 1 << (i % 8);
            }
        }

        Buffer::from_u8_slice(&bytes)
    }
}

fn is_null_bit(buffer: &Buffer, idx: usize) -> bool {
    let byte = buffer.as_u8_slice()[idx / 8];
    // (1 << (idx % 8)) -> mask with idx set
    (byte & (1 << (idx % 8))) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_and_read_bools() {
        // Create bitmap: [true, false, true, true, false, false, true, false]
        let bools = vec![true, false, true, true, false, false, true, false];
        let null_buffer = NullBuffer::from_bools(&bools);

        // Verify each position
        assert!(!null_buffer.is_null(0)); // true -> not null
        assert!(null_buffer.is_null(1)); // false -> null
        assert!(!null_buffer.is_null(2)); // true -> not null
        assert!(!null_buffer.is_null(3)); // true -> not null
        assert!(null_buffer.is_null(4)); // false -> null
        assert!(null_buffer.is_null(5)); // false -> null
        assert!(!null_buffer.is_null(6)); // true -> not null
        assert!(null_buffer.is_null(7)); // false -> null

        // Verify null_count
        assert_eq!(null_buffer.null_count(), 4);
    }

    #[test]
    fn test_null_buffer_all_valid() {
        let bools = vec![true, true, true];
        let null_buffer = NullBuffer::from_bools(&bools);

        assert_eq!(null_buffer.null_count(), 0);
        assert!(!null_buffer.is_null(0));
        assert!(!null_buffer.is_null(1));
        assert!(!null_buffer.is_null(2));
    }

    #[test]
    fn test_null_buffer_all_null() {
        let bools = vec![false, false, false];
        let null_buffer = NullBuffer::from_bools(&bools);

        assert_eq!(null_buffer.null_count(), 3);
        assert!(null_buffer.is_null(0));
        assert!(null_buffer.is_null(1));
        assert!(null_buffer.is_null(2));
    }
}
