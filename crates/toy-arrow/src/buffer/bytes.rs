use std::{
    alloc::{Layout, dealloc},
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
    /// Number of bytes in this allocation.
    len: usize,
    /// The Layout used for allocation. MUST be stored and reused for deallocation,
    /// as dealloc() requires the exact same Layout that was passed to alloc().
    layout: Layout,
}

impl Bytes {
    /// Creates a Bytes from an existing allocation.
    ///
    /// ## Safety contract
    /// The caller must ensure:
    /// - `ptr` points to memory allocated with the given `layout`
    /// - The memory will not be freed elsewhere (caller must forget the original owner)
    pub unsafe fn new(ptr: NonNull<u8>, len: usize, layout: Layout) -> Self {
        Self { ptr, len, layout }
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

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Drop for Bytes {
    fn drop(&mut self) {
        // Skip deallocation for zero-sized layouts.
        // Calling dealloc() with a zero-sized layout is undefined behavior.
        if self.layout.size() == 0 {
            return;
        }

        // SAFETY: We allocated this memory with the stored layout, and we're the
        // sole owner (guaranteed by Arc). Using the same layout for dealloc is required.
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}
