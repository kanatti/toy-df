// Placeholder for toy-arrow implementation

use std::marker::PhantomData;

pub mod bit_util;
pub mod buffer;
pub mod bytes;
pub mod native;

pub use buffer::{Buffer, NullBuffer};

pub struct PrimitiveArray<T> {
    values: Buffer,
    nulls: Option<NullBuffer>,
    len: usize,
    _phantom: PhantomData<T>,
}
