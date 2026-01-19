use std::marker::PhantomData;

use crate::{
    Buffer, NullBuffer, buffer::ScalarBuffer, data::ArrayData, native::NativeType,
    schema::PrimitiveType,
};

pub struct PrimitiveArray<T: PrimitiveType> {
    data: ArrayData,
    _phantom: PhantomData<T>,
}

impl<T: PrimitiveType> PrimitiveArray<T> {
    pub fn new(values: Buffer, nulls: Option<NullBuffer>) -> Self {
        let len = values.len() / T::Native::get_byte_width();
        let data = ArrayData::new(T::DATA_TYPE, 0, len, vec![values], nulls);
        Self {
            data,
            _phantom: PhantomData,
        }
    }

    pub fn value(&self, idx: usize) -> Option<T::Native> {
        if self.is_null(idx) {
            return None;
        }

        let buffer = self.data.buffers().get(0)?;
        let scalar_buffer: ScalarBuffer<T::Native> = buffer.clone().into();
        Some(scalar_buffer[idx])
    }

    pub fn is_null(&self, idx: usize) -> bool {
        match self.data.nulls() {
            None => false,
            Some(nulls) => nulls.is_null(idx),
        }
    }

    pub fn slice(&self, offset: usize, len: usize) -> Self {
        todo!()
    }
}
