// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::builder::BooleanBuilder;
use crate::iterator::BooleanIter;
use crate::raw_pointer::RawPtrBox;
use crate::{print_long_array, Array, ArrayAccessor};
use arrow_buffer::{bit_util, Buffer, MutableBuffer};
use arrow_data::ArrayData;
use arrow_schema::DataType;
use std::any::Any;

/// Array of bools
///
/// # Example
///
/// ```
///     use arrow_array::{Array, BooleanArray};
///     let arr = BooleanArray::from(vec![Some(false), Some(true), None, Some(true)]);
///     assert_eq!(4, arr.len());
///     assert_eq!(1, arr.null_count());
///     assert!(arr.is_valid(0));
///     assert!(!arr.is_null(0));
///     assert_eq!(false, arr.value(0));
///     assert!(arr.is_valid(1));
///     assert!(!arr.is_null(1));
///     assert_eq!(true, arr.value(1));
///     assert!(!arr.is_valid(2));
///     assert!(arr.is_null(2));
///     assert!(arr.is_valid(3));
///     assert!(!arr.is_null(3));
///     assert_eq!(true, arr.value(3));
/// ```
///
/// Using `from_iter`
/// ```
///     use arrow_array::{Array, BooleanArray};
///     let v = vec![Some(false), Some(true), Some(false), Some(true)];
///     let arr = v.into_iter().collect::<BooleanArray>();
///     assert_eq!(4, arr.len());
///     assert_eq!(0, arr.offset());
///     assert_eq!(0, arr.null_count());
///     assert!(arr.is_valid(0));
///     assert_eq!(false, arr.value(0));
///     assert!(arr.is_valid(1));
///     assert_eq!(true, arr.value(1));
///     assert!(arr.is_valid(2));
///     assert_eq!(false, arr.value(2));
///     assert!(arr.is_valid(3));
///     assert_eq!(true, arr.value(3));
/// ```
pub struct BooleanArray {
    data: ArrayData,
    /// Pointer to the value array. The lifetime of this must be <= to the value buffer
    /// stored in `data`, so it's safe to store.
    raw_values: RawPtrBox<u8>,
}

impl std::fmt::Debug for BooleanArray {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "BooleanArray\n[\n")?;
        print_long_array(self, f, |array, index, f| {
            std::fmt::Debug::fmt(&array.value(index), f)
        })?;
        write!(f, "]")
    }
}

impl BooleanArray {
    /// Returns the length of this array.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns whether this array is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    // Returns a new boolean array builder
    pub fn builder(capacity: usize) -> BooleanBuilder {
        BooleanBuilder::with_capacity(capacity)
    }

    /// Returns a `Buffer` holding all the values of this array.
    ///
    /// Note this doesn't take the offset of this array into account.
    pub fn values(&self) -> &Buffer {
        &self.data.buffers()[0]
    }

    /// Returns the boolean value at index `i`.
    ///
    /// # Safety
    /// This doesn't check bounds, the caller must ensure that index < self.len()
    pub unsafe fn value_unchecked(&self, i: usize) -> bool {
        let offset = i + self.offset();
        bit_util::get_bit_raw(self.raw_values.as_ptr(), offset)
    }

    /// Returns the boolean value at index `i`.
    /// # Panics
    /// Panics if index `i` is out of bounds
    pub fn value(&self, i: usize) -> bool {
        assert!(
            i < self.len(),
            "Trying to access an element at index {} from a BooleanArray of length {}",
            i,
            self.len()
        );
        // Safety:
        // `i < self.len()
        unsafe { self.value_unchecked(i) }
    }

    /// Returns an iterator that returns the values of `array.value(i)` for an iterator with each element `i`
    pub fn take_iter<'a>(
        &'a self,
        indexes: impl Iterator<Item = Option<usize>> + 'a,
    ) -> impl Iterator<Item = Option<bool>> + 'a {
        indexes.map(|opt_index| opt_index.map(|index| self.value(index)))
    }

    /// Returns an iterator that returns the values of `array.value(i)` for an iterator with each element `i`
    /// # Safety
    ///
    /// caller must ensure that the offsets in the iterator are less than the array len()
    pub unsafe fn take_iter_unchecked<'a>(
        &'a self,
        indexes: impl Iterator<Item = Option<usize>> + 'a,
    ) -> impl Iterator<Item = Option<bool>> + 'a {
        indexes.map(|opt_index| opt_index.map(|index| self.value_unchecked(index)))
    }
}

impl Array for BooleanArray {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn data(&self) -> &ArrayData {
        &self.data
    }

    fn into_data(self) -> ArrayData {
        self.into()
    }
}

impl<'a> ArrayAccessor for &'a BooleanArray {
    type Item = bool;

    fn value(&self, index: usize) -> Self::Item {
        BooleanArray::value(self, index)
    }

    unsafe fn value_unchecked(&self, index: usize) -> Self::Item {
        BooleanArray::value_unchecked(self, index)
    }
}

impl From<Vec<bool>> for BooleanArray {
    fn from(data: Vec<bool>) -> Self {
        let mut mut_buf = MutableBuffer::new_null(data.len());
        {
            let mut_slice = mut_buf.as_slice_mut();
            for (i, b) in data.iter().enumerate() {
                if *b {
                    bit_util::set_bit(mut_slice, i);
                }
            }
        }
        let array_data = ArrayData::builder(DataType::Boolean)
            .len(data.len())
            .add_buffer(mut_buf.into());

        let array_data = unsafe { array_data.build_unchecked() };
        BooleanArray::from(array_data)
    }
}

impl From<Vec<Option<bool>>> for BooleanArray {
    fn from(data: Vec<Option<bool>>) -> Self {
        data.iter().collect()
    }
}

impl From<ArrayData> for BooleanArray {
    fn from(data: ArrayData) -> Self {
        assert_eq!(
            data.buffers().len(),
            1,
            "BooleanArray data should contain a single buffer only (values buffer)"
        );
        let ptr = data.buffers()[0].as_ptr();
        Self {
            data,
            raw_values: unsafe { RawPtrBox::new(ptr) },
        }
    }
}

impl From<BooleanArray> for ArrayData {
    fn from(array: BooleanArray) -> Self {
        array.data
    }
}

impl<'a> IntoIterator for &'a BooleanArray {
    type Item = Option<bool>;
    type IntoIter = BooleanIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BooleanIter::<'a>::new(self)
    }
}

impl<'a> BooleanArray {
    /// constructs a new iterator
    pub fn iter(&'a self) -> BooleanIter<'a> {
        BooleanIter::<'a>::new(self)
    }
}

impl<Ptr: std::borrow::Borrow<Option<bool>>> FromIterator<Ptr> for BooleanArray {
    fn from_iter<I: IntoIterator<Item = Ptr>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (_, data_len) = iter.size_hint();
        let data_len = data_len.expect("Iterator must be sized"); // panic if no upper bound.

        let num_bytes = bit_util::ceil(data_len, 8);
        let mut null_builder = MutableBuffer::from_len_zeroed(num_bytes);
        let mut val_builder = MutableBuffer::from_len_zeroed(num_bytes);

        let data = val_builder.as_slice_mut();

        let null_slice = null_builder.as_slice_mut();
        iter.enumerate().for_each(|(i, item)| {
            if let Some(a) = item.borrow() {
                bit_util::set_bit(null_slice, i);
                if *a {
                    bit_util::set_bit(data, i);
                }
            }
        });

        let data = unsafe {
            ArrayData::new_unchecked(
                DataType::Boolean,
                data_len,
                None,
                Some(null_builder.into()),
                0,
                vec![val_builder.into()],
                vec![],
            )
        };
        BooleanArray::from(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_fmt_debug() {
        let arr = BooleanArray::from(vec![true, false, false]);
        assert_eq!(
            "BooleanArray\n[\n  true,\n  false,\n  false,\n]",
            format!("{:?}", arr)
        );
    }

    #[test]
    fn test_boolean_with_null_fmt_debug() {
        let mut builder = BooleanArray::builder(3);
        builder.append_value(true);
        builder.append_null();
        builder.append_value(false);
        let arr = builder.finish();
        assert_eq!(
            "BooleanArray\n[\n  true,\n  null,\n  false,\n]",
            format!("{:?}", arr)
        );
    }

    #[test]
    fn test_boolean_array_from_vec() {
        let buf = Buffer::from([10_u8]);
        let arr = BooleanArray::from(vec![false, true, false, true]);
        assert_eq!(&buf, arr.values());
        assert_eq!(4, arr.len());
        assert_eq!(0, arr.offset());
        assert_eq!(0, arr.null_count());
        for i in 0..4 {
            assert!(!arr.is_null(i));
            assert!(arr.is_valid(i));
            assert_eq!(i == 1 || i == 3, arr.value(i), "failed at {}", i)
        }
    }

    #[test]
    fn test_boolean_array_from_vec_option() {
        let buf = Buffer::from([10_u8]);
        let arr = BooleanArray::from(vec![Some(false), Some(true), None, Some(true)]);
        assert_eq!(&buf, arr.values());
        assert_eq!(4, arr.len());
        assert_eq!(0, arr.offset());
        assert_eq!(1, arr.null_count());
        for i in 0..4 {
            if i == 2 {
                assert!(arr.is_null(i));
                assert!(!arr.is_valid(i));
            } else {
                assert!(!arr.is_null(i));
                assert!(arr.is_valid(i));
                assert_eq!(i == 1 || i == 3, arr.value(i), "failed at {}", i)
            }
        }
    }

    #[test]
    fn test_boolean_array_from_iter() {
        let v = vec![Some(false), Some(true), Some(false), Some(true)];
        let arr = v.into_iter().collect::<BooleanArray>();
        assert_eq!(4, arr.len());
        assert_eq!(0, arr.offset());
        assert_eq!(0, arr.null_count());
        assert!(arr.data().null_buffer().is_none());
        for i in 0..3 {
            assert!(!arr.is_null(i));
            assert!(arr.is_valid(i));
            assert_eq!(i == 1 || i == 3, arr.value(i), "failed at {}", i)
        }
    }

    #[test]
    fn test_boolean_array_from_nullable_iter() {
        let v = vec![Some(true), None, Some(false), None];
        let arr = v.into_iter().collect::<BooleanArray>();
        assert_eq!(4, arr.len());
        assert_eq!(0, arr.offset());
        assert_eq!(2, arr.null_count());
        assert!(arr.data().null_buffer().is_some());

        assert!(arr.is_valid(0));
        assert!(arr.is_null(1));
        assert!(arr.is_valid(2));
        assert!(arr.is_null(3));

        assert!(arr.value(0));
        assert!(!arr.value(2));
    }

    #[test]
    fn test_boolean_array_builder() {
        // Test building a boolean array with ArrayData builder and offset
        // 000011011
        let buf = Buffer::from([27_u8]);
        let buf2 = buf.clone();
        let data = ArrayData::builder(DataType::Boolean)
            .len(5)
            .offset(2)
            .add_buffer(buf)
            .build()
            .unwrap();
        let arr = BooleanArray::from(data);
        assert_eq!(&buf2, arr.values());
        assert_eq!(5, arr.len());
        assert_eq!(2, arr.offset());
        assert_eq!(0, arr.null_count());
        for i in 0..3 {
            assert_eq!(i != 0, arr.value(i), "failed at {}", i);
        }
    }

    #[test]
    #[should_panic(
        expected = "Trying to access an element at index 4 from a BooleanArray of length 3"
    )]
    fn test_fixed_size_binary_array_get_value_index_out_of_bound() {
        let v = vec![Some(true), None, Some(false)];
        let array = v.into_iter().collect::<BooleanArray>();

        array.value(4);
    }

    #[test]
    #[should_panic(expected = "BooleanArray data should contain a single buffer only \
                               (values buffer)")]
    // Different error messages, so skip for now
    // https://github.com/apache/arrow-rs/issues/1545
    #[cfg(not(feature = "force_validate"))]
    fn test_boolean_array_invalid_buffer_len() {
        let data = unsafe {
            ArrayData::builder(DataType::Boolean)
                .len(5)
                .build_unchecked()
        };
        drop(BooleanArray::from(data));
    }
}
