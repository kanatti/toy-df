//! Buffer types for Arrow arrays with clear layering and responsibilities.
//!
//! ## Architecture
//!
//! ```text
//! MutableBuffer ──into_buffer()──→ Buffer(Arc<Bytes>)
//!                                       ↓
//!                      ScalarBuffer<T>, BooleanBuffer, NullBuffer
//! ```
//!
//! ## Details
//!
//! - **MutableBuffer**: Build phase. Owns memory directly, growable. Converts to Buffer when done.
//! - **Buffer**: Immutable, contains `Arc<Bytes>`. Cheap clone (refcount increment). Zero-copy slicing.
//! - **Bytes**: Raw allocation with guaranteed alignment. Handles deallocation on drop.
//! - **ScalarBuffer<T>**: Type-safe view over Buffer. Enforces alignment, provides `&[T]`.
//! - **BooleanBuffer**: Bit-packed booleans (8 per byte) over Buffer.
//! - **NullBuffer**: Bit-packed validity bitmap over BooleanBuffer, cached null_count.

mod boolean;
mod bytes;
mod immutable;
mod mutable;
mod null;
mod scalar;

pub use boolean::BooleanBuffer;
pub use bytes::Bytes;
pub use immutable::Buffer;
pub use mutable::MutableBuffer;
pub use null::NullBuffer;
pub use scalar::ScalarBuffer;
