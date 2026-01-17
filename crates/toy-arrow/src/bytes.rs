use std::{
    alloc::{Layout, alloc, dealloc},
    ptr::NonNull,
};

/// The actual memory allocation, shared by multiple Buffers via Arc.
///
/// ## Why custom allocation instead of Vec<u8>?
///
/// We need guaranteed alignment for typed access. While Vec<u8> may happen to be
/// aligned (allocators often align to 8/16 bytes), Rust only *guarantees* 1-byte
/// alignment for Vec<u8>.
///
/// To safely reinterpret bytes as i32/f64/etc, we need explicit alignment.
/// Creating a misaligned reference (e.g., &[i32] at address 0x1001) is undefined
/// behavior in Rust, even on CPUs that tolerate misaligned access.
///
/// ## Memory ownership
///
/// Bytes owns the allocation and frees it on drop. The `layout` field stores
/// the exact Layout used for allocation, which is required for correct deallocation.
pub struct Bytes {
    /// Pointer to allocated memory. NonNull provides null-safety and covariance.
    ptr: NonNull<u8>,
    /// Number of bytes currently in use (may be less than capacity).
    length: usize,
    /// Total allocated bytes. Currently unused but kept for potential future use
    /// (e.g., growing buffers, debugging).
    #[allow(dead_code)]
    capacity: usize,
    /// The Layout used for allocation. MUST be stored and reused for deallocation,
    /// as dealloc() requires the exact same Layout that was passed to alloc().
    layout: Layout,
}

impl Bytes {
    /// Allocates a new buffer with the given capacity and alignment.
    pub fn new(capacity: usize, alignment: usize) -> Self {
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

    /// Creates a Bytes from an existing allocation (e.g., from a Vec).
    ///
    /// ## Safety contract
    /// The caller must ensure:
    /// - `ptr` points to memory allocated with the given `layout`
    /// - The memory will not be freed elsewhere (caller must forget the original owner)
    pub fn from_raw_parts(ptr: NonNull<u8>, length: usize, layout: Layout) -> Self {
        Self {
            ptr,
            length,
            capacity: layout.size(),
            layout,
        }
    }

    /// Returns a pointer offset by `n` bytes from the start.
    pub fn offset_ptr(&self, n: usize) -> *const u8 {
        // SAFETY: Callers ensure n is within bounds
        unsafe { self.ptr.as_ptr().add(n) }
    }

    /// Returns the raw pointer to the start of the allocation.
    pub fn ptr(&self) -> NonNull<u8> {
        self.ptr
    }

    pub fn set_length(&mut self, length: usize) {
        self.length = length;
    }
}

impl Drop for Bytes {
    fn drop(&mut self) {
        // SAFETY: We allocated this memory with the stored layout, and we're the
        // sole owner (guaranteed by Arc). Using the same layout for dealloc is required.
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bytes = Bytes::new(100, 8);
        assert_eq!(bytes.capacity, 100);
        assert_eq!(bytes.length, 0);
    }

    #[test]
    fn test_alignment() {
        let bytes = Bytes::new(100, 8);
        let addr = bytes.ptr.as_ptr() as usize;
        assert_eq!(addr % 8, 0, "Pointer should be 8-byte aligned");
    }
}
