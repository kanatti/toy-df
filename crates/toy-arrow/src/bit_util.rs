use crate::Buffer;

/// Returns true if the bit at `idx` is set (1), false if unset (0).
///
/// ```text
/// To get bit 5 in byte 0b01001101:
///   mask = 1 << 5 = 0b00100000
///   byte & mask   = 0b00000000  (bit 5 is 0)
///   result != 0   → false
/// ```
pub fn get_bit(buffer: &Buffer, idx: usize) -> bool {
    let byte = buffer.as_u8_slice()[idx / 8];
    let mask = 1 << (idx % 8);
    (byte & mask) != 0
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
pub fn pack_bools(bools: &[bool]) -> Buffer {
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
