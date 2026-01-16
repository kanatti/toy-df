use crate::{Buffer, bit_util};

/// A bit-packed buffer of boolean values.
///
/// Stores booleans efficiently using 1 bit per value instead of 1 byte,
/// giving 8x memory savings. For example, 1 million booleans use ~125KB
/// instead of 1MB.
///
/// ## Difference from NullBuffer
///
/// Both are bit-packed, but serve different purposes:
/// - `BooleanBuffer`: stores actual boolean **values** (the data itself)
/// - `NullBuffer`: stores **validity** (is each slot null or not?)
///
/// A `BooleanArray` would use both: a `BooleanBuffer` for the true/false values,
/// and optionally a `NullBuffer` to track which positions are null.
///
/// In fact, `NullBuffer` wraps a `BooleanBuffer` internally and just adds
/// the cached `null_count` and inverted semantics.
///
/// ## Example
/// ```text
/// Values: [true, false, true, true, false]
/// Packed:  0b00001101 (single byte, bits 0-4 used, 5-7 are padding)
///            ^^^^^ meaningful bits
/// ```
pub struct BooleanBuffer {
    buffer: Buffer,
    /// Number of logical boolean values (bits), not bytes.
    ///
    /// Needed because the buffer stores whole bytes, but the actual data
    /// may not fill the last byte. For example, 5 booleans need 1 byte,
    /// but only 5 of the 8 bits are meaningful.
    len: usize,
}

impl BooleanBuffer {
    /// Creates a BooleanBuffer from a raw buffer and logical length.
    ///
    /// The buffer should contain bit-packed booleans. `len` is the number
    /// of boolean values, not bytes.
    pub fn new(buffer: Buffer, len: usize) -> Self {
        Self { buffer, len }
    }

    /// Returns the number of boolean values in this buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the boolean value at the given index.
    ///
    /// ## Panics
    /// Panics if `idx >= len`.
    pub fn value(&self, idx: usize) -> bool {
        assert!(idx < self.len, "index out of bounds");
        bit_util::get_bit(&self.buffer, idx)
    }

    /// Creates a BooleanBuffer from a slice of booleans.
    ///
    /// Packs the booleans into bytes using LSB-first ordering (Arrow convention).
    pub fn from_bools(bools: &[bool]) -> Self {
        let buffer = bit_util::pack_bools(bools);
        Self::new(buffer, bools.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bools() {
        let bools = vec![true, false, true, true, false, false, true, false];
        let buffer = BooleanBuffer::from_bools(&bools);

        assert_eq!(buffer.value(0), true);
        assert_eq!(buffer.value(1), false);
        assert_eq!(buffer.value(2), true);
        assert_eq!(buffer.value(3), true);
        assert_eq!(buffer.value(4), false);
        assert_eq!(buffer.value(5), false);
        assert_eq!(buffer.value(6), true);
        assert_eq!(buffer.value(7), false);
    }

    #[test]
    fn test_len() {
        let buffer = BooleanBuffer::from_bools(&[true, false, true]);
        assert_eq!(buffer.len(), 3);

        let buffer = BooleanBuffer::from_bools(&[]);
        assert_eq!(buffer.len(), 0);

        // 10 bools needs 2 bytes, but len should be 10
        let buffer = BooleanBuffer::from_bools(&[true; 10]);
        assert_eq!(buffer.len(), 10);
    }

    #[test]
    fn test_all_true() {
        let buffer = BooleanBuffer::from_bools(&[true, true, true, true]);
        for i in 0..4 {
            assert_eq!(buffer.value(i), true);
        }
    }

    #[test]
    fn test_all_false() {
        let buffer = BooleanBuffer::from_bools(&[false, false, false, false]);
        for i in 0..4 {
            assert_eq!(buffer.value(i), false);
        }
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_out_of_bounds() {
        let buffer = BooleanBuffer::from_bools(&[true, false, true]);
        buffer.value(3); // len is 3, so index 3 is out of bounds
    }

    #[test]
    fn test_partial_byte() {
        // 5 booleans = 1 byte with 3 padding bits
        let bools = vec![true, false, true, false, true];
        let buffer = BooleanBuffer::from_bools(&bools);

        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.value(0), true);
        assert_eq!(buffer.value(1), false);
        assert_eq!(buffer.value(2), true);
        assert_eq!(buffer.value(3), false);
        assert_eq!(buffer.value(4), true);
    }
}
