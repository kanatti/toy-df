use crate::buffer::BooleanBuffer;

/// A validity bitmap indicating which values in an array are null.
///
/// Wraps a [`BooleanBuffer`] and adds a cached null count. The underlying storage
/// is identical - both are bit-packed. NullBuffer just provides:
/// 1. Inverted semantics: `is_null()` returns true when bit is 0
/// 2. Cached `null_count` for O(1) access (query engines check this constantly)
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
    buffer: BooleanBuffer,
    /// Cached count of null values. Computing this requires scanning all bits O(n),
    /// so we cache it at construction for O(1) access. Query engines frequently
    /// check if arrays have nulls, making this optimization worthwhile.
    null_count: usize,
}

impl NullBuffer {
    /// Creates a NullBuffer from a BooleanBuffer.
    /// Scans the buffer to compute and cache the null count.
    pub fn new(buffer: BooleanBuffer) -> Self {
        let mut null_count = 0;
        for i in 0..buffer.len() {
            if !buffer.value(i) {
                null_count += 1;
            }
        }

        Self { buffer, null_count }
    }

    /// Creates a NullBuffer from a slice of booleans.
    /// `true` means valid, `false` means null.
    pub fn from_bools(bools: &[bool]) -> Self {
        let buffer = BooleanBuffer::from_bools(bools);
        Self::new(buffer)
    }

    /// Returns the cached null count in O(1).
    pub fn null_count(&self) -> usize {
        self.null_count
    }

    /// Returns the number of logical elements (not bytes).
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns true if the value at `idx` is null (bit is 0).
    pub fn is_null(&self, idx: usize) -> bool {
        !self.buffer.value(idx)
    }

    pub fn slice(&self, offset: usize, len: usize) -> Self {
        let buffer = self.buffer.slice(offset, len);
        // Recompute null_count for the sliced region
        let mut null_count = 0;
        for i in 0..len {
            if !buffer.value(i) {
                null_count += 1;
            }
        }
        Self { buffer, null_count }
    }
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

    #[test]
    fn test_null_buffer_slice() {
        // [valid, null, valid, null, valid]
        let bools = vec![true, false, true, false, true];
        let null_buffer = NullBuffer::from_bools(&bools);
        assert_eq!(null_buffer.null_count(), 2);

        // slice(1, 3) → [null, valid, null]
        let sliced = null_buffer.slice(1, 3);
        assert_eq!(sliced.len(), 3);
        assert_eq!(sliced.null_count(), 2);
        assert!(sliced.is_null(0));  // was index 1 (null)
        assert!(!sliced.is_null(1)); // was index 2 (valid)
        assert!(sliced.is_null(2));  // was index 3 (null)
    }

    #[test]
    fn test_null_buffer_slice_no_nulls() {
        // [valid, null, valid, valid, valid, null]
        let bools = vec![true, false, true, true, true, false];
        let null_buffer = NullBuffer::from_bools(&bools);

        // slice(2, 3) → [valid, valid, valid]
        let sliced = null_buffer.slice(2, 3);
        assert_eq!(sliced.len(), 3);
        assert_eq!(sliced.null_count(), 0);
    }
}
