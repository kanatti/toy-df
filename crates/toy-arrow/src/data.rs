use crate::{Buffer, NullBuffer, data, schema::DataType};

pub struct ArrayData {
    data_type: DataType,
    offset: usize,
    len: usize,
    buffers: Vec<Buffer>,
    nulls: Option<NullBuffer>,
}

impl ArrayData {
    pub fn new(
        data_type: DataType,
        offset: usize,
        len: usize,
        buffers: Vec<Buffer>,
        nulls: Option<NullBuffer>,
    ) -> Self {
        ArrayData {
            data_type,
            offset,
            len,
            buffers,
            nulls,
        }
    }

    pub fn nulls(&self) -> Option<&NullBuffer> {
        self.nulls.as_ref()
    }

    pub fn buffers(&self) -> &Vec<Buffer> {
        &self.buffers
    }
}
