// Placeholder for toy-arrow implementation

use std::marker::PhantomData;

pub mod buffer;

pub use buffer::Buffer;

pub struct NullBuffer {}

pub struct PrimitiveArray<T> {
    values: Buffer,
    nulls: Option<NullBuffer>,
    len: usize,
    _phantom: PhantomData<T>,
}
