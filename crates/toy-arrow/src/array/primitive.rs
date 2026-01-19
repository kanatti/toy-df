use crate::{
    NullBuffer,
    buffer::ScalarBuffer,
    schema::{DataType, PrimitiveType, primitive},
};

// Type aliases for common primitive arrays
pub type Int8Array = PrimitiveArray<primitive::Int8Type>;
pub type Int16Array = PrimitiveArray<primitive::Int16Type>;
pub type Int32Array = PrimitiveArray<primitive::Int32Type>;
pub type Int64Array = PrimitiveArray<primitive::Int64Type>;
pub type UInt8Array = PrimitiveArray<primitive::UInt8Type>;
pub type UInt16Array = PrimitiveArray<primitive::UInt16Type>;
pub type UInt32Array = PrimitiveArray<primitive::UInt32Type>;
pub type UInt64Array = PrimitiveArray<primitive::UInt64Type>;
pub type Float32Array = PrimitiveArray<primitive::Float32Type>;
pub type Float64Array = PrimitiveArray<primitive::Float64Type>;

pub struct PrimitiveArray<T: PrimitiveType> {
    data_type: DataType,
    values: ScalarBuffer<T::Native>,
    nulls: Option<NullBuffer>,
}

impl<T: PrimitiveType> PrimitiveArray<T> {
    pub fn new(values: ScalarBuffer<T::Native>, nulls: Option<NullBuffer>) -> Self {
        Self {
            data_type: T::DATA_TYPE,
            values,
            nulls,
        }
    }

    /// Returns the length of this array
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns the values buffer
    #[inline]
    pub fn values(&self) -> &ScalarBuffer<T::Native> {
        &self.values
    }

    /// Returns the primitive value at index `i`.
    ///
    /// Note: this does not check for nulls. The value is arbitrary
    /// if `is_null(i)` returns true.
    #[inline]
    pub fn value(&self, idx: usize) -> T::Native {
        self.values[idx]
    }

    /// Returns true if the value at index `i` is null
    pub fn is_null(&self, idx: usize) -> bool {
        match &self.nulls {
            None => false,
            Some(nulls) => nulls.is_null(idx),
        }
    }

    pub fn slice(&self, offset: usize, len: usize) -> Self {
        Self {
            data_type: self.data_type.clone(),
            values: self.values.slice(offset, len),
            nulls: self.nulls.as_ref().map(|n| n.slice(offset, len)),
        }
    }

    pub fn from_options(_opts: Vec<Option<T::Native>>) -> Self {
        todo!("Need Default bound or zero value for null slots")
    }
}

impl<T: PrimitiveType> FromIterator<T::Native> for PrimitiveArray<T> {
    fn from_iter<I: IntoIterator<Item = T::Native>>(iter: I) -> Self {
        let values: Vec<T::Native> = iter.into_iter().collect();
        PrimitiveArray {
            data_type: T::DATA_TYPE,
            values: ScalarBuffer::from(values),
            nulls: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_iter() {
        let arr: Int32Array = vec![1, 2, 3, 4, 5].into_iter().collect();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr.value(0), 1);
        assert_eq!(arr.value(4), 5);
    }

    #[test]
    fn test_value_access() {
        let arr: Int32Array = vec![10, 20, 30].into_iter().collect();
        assert_eq!(arr.value(0), 10);
        assert_eq!(arr.value(1), 20);
        assert_eq!(arr.value(2), 30);
    }

    #[test]
    fn test_no_nulls() {
        let arr: Int32Array = vec![1, 2, 3].into_iter().collect();
        assert!(!arr.is_null(0));
        assert!(!arr.is_null(1));
        assert!(!arr.is_null(2));
    }

    #[test]
    fn test_with_nulls() {
        let values = ScalarBuffer::from(vec![10, 0, 30]);
        let nulls = NullBuffer::from_bools(&[true, false, true]);
        let arr = Int32Array::new(values, Some(nulls));

        assert_eq!(arr.len(), 3);
        assert!(!arr.is_null(0));
        assert!(arr.is_null(1));
        assert!(!arr.is_null(2));
        assert_eq!(arr.value(0), 10);
        assert_eq!(arr.value(2), 30);
    }

    #[test]
    fn test_slice() {
        let arr: Int32Array = vec![10, 20, 30, 40, 50].into_iter().collect();
        let sliced = arr.slice(1, 3);

        assert_eq!(sliced.len(), 3);
        assert_eq!(sliced.value(0), 20);
        assert_eq!(sliced.value(1), 30);
        assert_eq!(sliced.value(2), 40);
    }

    #[test]
    fn test_slice_with_nulls() {
        // [10, null, 30, null, 50]
        let values = ScalarBuffer::from(vec![10, 0, 30, 0, 50]);
        let nulls = NullBuffer::from_bools(&[true, false, true, false, true]);
        let arr = Int32Array::new(values, Some(nulls));

        // slice(1, 3) → [null, 30, null]
        let sliced = arr.slice(1, 3);
        assert_eq!(sliced.len(), 3);
        assert!(sliced.is_null(0));
        assert!(!sliced.is_null(1));
        assert!(sliced.is_null(2));
        assert_eq!(sliced.value(1), 30);
    }
}
