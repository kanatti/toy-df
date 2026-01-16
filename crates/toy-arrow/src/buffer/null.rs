use crate::Buffer;

/// A validity bitmap indicating which values in an array are null.
///
/// Arrow uses bit-packing for space efficiency: 1 bit per element instead of 1 byte.
/// This gives 8x memory savings (e.g., 125 bytes for 1000 elements vs 1000 bytes).
///
/// ## Convention
/// - Bit = 1 (true) → value is **valid** (not null)
/// - Bit = 0 (false) → value is **null**
///
/// ## Example
/// ```text
/// Values:    [10, NULL, 30, NULL, 50]
/// Validity:  [ 1,    0,  1,    0,  1]  (as bits)
/// Packed:    0b00010101 = 21          (single byte)
/// ```
pub struct NullBuffer {
    buffer: Buffer,
    /// Number of logical elements (bits), not bytes.
    len: usize,
    /// Cached count of null values. Computing this requires scanning all bits O(n),
    /// so we cache it at construction for O(1) access. Query engines frequently
    /// check if arrays have nulls, making this optimization worthwhile.
    null_count: usize,
}

impl NullBuffer {
    /// Creates a NullBuffer from a raw buffer and logical length.
    /// Scans the buffer to compute and cache the null count.
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

    /// Creates a NullBuffer from a slice of booleans.
    /// `true` means valid, `false` means null.
    pub fn from_bools(bools: &[bool]) -> Self {
        let buffer = Self::pack_bools(bools);
        Self::new(buffer, bools.len())
    }

    /// Returns the cached null count in O(1).
    pub fn null_count(&self) -> usize {
        self.null_count
    }

    /// Returns the number of logical elements (not bytes).
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the value at `idx` is null (bit is 0).
    pub fn is_null(&self, idx: usize) -> bool {
        assert!(idx < self.len, "index out of bounds");
        is_null_bit(&self.buffer, idx)
    }

    /// Packs booleans into bytes, with least-significant bit first (LSB numbering).
    ///
    /// Example: `[true, false, true, true, false, false, true, false]`
    /// ```text
    /// Index:      0     1     2     3     4     5     6     7
    /// Value:      1     0     1     1     0     0     1     0
    /// Bit pos:    0     1     2     3     4     5     6     7  (LSB to MSB)
    /// ```
    /// Byte = 0b01001101 = 77
    ///
    /// Why LSB first? Arrow spec uses this convention. Bit 0 is the least significant bit.
    fn pack_bools(bools: &[bool]) -> Buffer {
        // Ceiling division: 8 bools need 1 byte, 9 bools need 2 bytes
        let num_bytes = (bools.len() + 7) / 8;
        let mut bytes = vec![0u8; num_bytes];

        for (i, &is_valid) in bools.iter().enumerate() {
            if is_valid {
                // i / 8      → which byte contains bit i
                // i % 8      → position within that byte (0-7)
                // 1 << (i%8) → mask with only that bit set
                // |=         → set the bit (OR preserves other bits)
                bytes[i / 8] |= 1 << (i % 8);
            }
            // If !is_valid, leave bit as 0 (null)
        }

        Buffer::from_u8_slice(&bytes)
    }
}

/// Checks if the bit at `idx` is 0 (null).
///
/// ```text
/// To check bit 5 in byte 0b01001101:
///   mask = 1 << 5 = 0b00100000
///   byte & mask   = 0b00000000  (bit 5 is 0, so result is 0)
///   result == 0   → true, it's null
/// ```
fn is_null_bit(buffer: &Buffer, idx: usize) -> bool {
    let byte = buffer.as_u8_slice()[idx / 8];
    let mask = 1 << (idx % 8);
    (byte & mask) == 0
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
