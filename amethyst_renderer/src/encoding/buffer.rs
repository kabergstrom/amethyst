use super::properties::{EncodedProp, EncodingValue, IterableEncoding};
use std::marker::PhantomData;

/// Trait that defines the encoding buffer writing stragety for a specified
/// shader layout.
/// Every encoder must push exactly one value per iterated entity to the buffer.
///
/// The encoding scheduler is free to implement it in any way that is appropriate
/// for given situation. For example, multiple `EncodeBuffer` views might use
/// the same underlying buffer, but write with a common stride and different offsets.
pub trait EncodeBuffer<T: EncodingValue> {
    /// Push encoded values to the buffer. Must be called exactly once for every entry
    /// in the provided encoding iterator.
    fn push(&mut self, data: T::Value);
}

struct BinBufferStride<'a> {
    begin: *mut u8,
    stride: isize,
    elem_count: isize,
    contiguous_count: usize,
    life: PhantomData<&'a mut u8>,
}

impl<'a> BinBufferStride<'a> {
    #[allow(dead_code)]
    /// TODO docs
    pub fn from_layout<'s>(
        slice: &'a mut [u8],
        layout: &EncodingLayout,
    ) -> impl Iterator<Item = BinBufferStride<'a>> + 's {
        let stride: usize = sizes.iter().sum();

        assert!(stride > 0 && slice.len() % stride == 0);

        let elem_count = slice.len() / stride;
        let mut_ptr = slice.as_mut_ptr();

        let mut start_offset: isize = 0;
        sizes.iter().map(move |size| {
            let begin = unsafe { mut_ptr.offset(start_offset) };
            let stride_struct = BinBufferStride {
                begin,
                stride: stride as isize,
                elem_count: elem_count as isize,
                contiguous_count: *size,
                life: PhantomData,
            };
            start_offset += *size as isize;
            stride_struct
        })
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut [u8] {
        debug_assert!((idx as isize) < self.elem_count);
        unsafe {
            let write_ptr = self.begin.offset(self.stride * idx as isize);
            std::slice::from_raw_parts_mut(write_ptr, self.contiguous_count)
        }
    }
}

pub struct StridedEncodeBuffer<'a, T: EncodingValue> {
    strides: &'a mut [BinBufferStride<'a>],
    write_idx: usize,
    marker: PhantomData<T>,
}

impl<'a, T: EncodingValue> StridedEncodeBuffer<'a, T> {
    /// TODO docs
    #[allow(dead_code)]
    fn new(strides: &'a mut [BinBufferStride<'a>]) -> Self {
        Self {
            strides,
            write_idx: 0,
            marker: PhantomData,
        }
    }
}

impl<'a, T: EncodingValue> EncodeBuffer<T> for StridedEncodeBuffer<'a, T> {
    fn push(&mut self, data: T::Value) {
        data.for_each(|idx, bytes| {
            let dst = self.strides[idx].get_mut(self.write_idx);
            dst.copy_from_slice(bytes);
        });
        self.write_idx += 1;
    }
}

/// TODO docs
pub struct EncodeBufferBuilder<'b> {
    _stride: BinBufferStride<'b>,
}

impl<'b> EncodeBufferBuilder<'b> {
    /// TODO docs
    pub fn create(_layout: &Vec<EncodedProp>, _raw_buffer: &mut [u8]) -> Self {
        unimplemented!()
    }

    /// TODO docs
    pub fn build<T: EncodingValue>(self) -> StridedEncodeBuffer<'b, T> {
        // StridedEncodeBuffer::new(self.stride)
        unimplemented!()
    }
}

// TODO: create buffer tree by sequential builders invocation
// create top-level builder with layout
// Use Rc<&[u8]> internally to push to
