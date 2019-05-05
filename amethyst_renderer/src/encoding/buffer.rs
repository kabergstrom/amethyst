use super::{EncodingLayout, EncodingValue, IterableEncoding};
use crate::encoding::properties::EncProperties;
use std::{
    cell::{RefCell, RefMut},
    marker::PhantomData,
};

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
    pub fn from_layout<'l>(
        slice: &'a mut [u8],
        layout: &'l EncodingLayout,
    ) -> impl Iterator<Item = BinBufferStride<'a>> + 'l {
        let stride: usize = layout.padded_size as usize;

        assert!(
            stride > 0 && slice.len() % stride == 0,
            "Buffer size {} must be a multiple of layout stride {}",
            slice.len(),
            stride
        );

        let elem_count = slice.len() / stride;
        let mut_ptr = slice.as_mut_ptr();

        // Let's assume that layout is well-formed and has no overlaps
        layout.props.iter().map(move |layout_prop| {
            let begin = unsafe { mut_ptr.offset(layout_prop.absolute_offset as isize) };
            let size = layout_prop.ubo_size();

            BinBufferStride {
                begin,
                stride: stride as isize,
                elem_count: elem_count as isize,
                contiguous_count: size,
                life: PhantomData,
            }
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

pub struct BufferWriter<'a, 'b, T: EncodingValue> {
    strides: Vec<RefMut<'b, BinBufferStride<'a>>>,
    write_idx: usize,
    marker: PhantomData<T>,
}

impl<'a, 'b, T: EncodingValue> BufferWriter<'a, 'b, T> {
    /// TODO docs
    #[allow(dead_code)]
    fn new(strides: Vec<RefMut<'b, BinBufferStride<'a>>>) -> Self {
        Self {
            strides,
            write_idx: 0,
            marker: PhantomData,
        }
    }
}

impl<'a, 'b, T: EncodingValue> EncodeBuffer<T> for BufferWriter<'a, 'b, T> {
    fn push(&mut self, data: T::Value) {
        data.for_each(|idx, bytes| {
            let dst = self.strides[idx].get_mut(self.write_idx);
            dst.copy_from_slice(bytes);
        });
        self.write_idx += 1;
    }
}

/// A builder for `BufferWriter`. Does the job of figuring out which strides should be written into in what order.
pub struct EncodeBufferBuilder<'a> {
    layout: EncodingLayout,
    strides: Vec<RefCell<BinBufferStride<'a>>>,
}

impl<'a> EncodeBufferBuilder<'a> {
    /// Create a `BufferWriteBuilder` with specific shader layout
    /// and buffer that's going to be written during encoding.
    pub fn create(layout: &EncodingLayout, raw_buffer: &'a mut [u8]) -> Self {
        Self {
            strides: BinBufferStride::from_layout(raw_buffer, layout)
                .map(RefCell::new)
                .collect(),
            layout: layout.clone(),
        }
    }

    /// Build a `BufferWriter` tailored for encoding of specific type.
    ///
    /// Works under an assumption that there is only one property of given name in a shader layout.
    pub fn build<'b, T: EncProperties>(&'b self) -> BufferWriter<'a, 'b, T::EncodedType> {
        let props_in_encoding_order = T::get_props();
        let stride_indices = props_in_encoding_order.map(|prop| {
            self.layout
                .props
                .iter()
                .position(|layout_prop| layout_prop.prop == prop)
                .expect("Trying to encode a prop that is not a part of provided layout")
        });

        let strides = stride_indices
            .map(|i| {
                self.strides[i]
                    .try_borrow_mut()
                    .ok()
                    .expect("Tries to encode the same data type multiple times")
            })
            .collect::<Vec<_>>();

        BufferWriter::new(strides)
    }
}
