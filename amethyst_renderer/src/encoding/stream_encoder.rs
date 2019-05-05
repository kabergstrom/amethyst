use super::{
    data::EncodingDef, properties::EncodingValue, EncProperties, EncodeBuffer, EncodeBufferBuilder,
    EncodedProp, EncodingData, FetchedData,
};
use amethyst_core::specs::SystemData;
use hibitset::BitSet;
use shred::Resources;
use std::{any::Any, marker::PhantomData};

/// A main trait that defines a strategy to encode specified stream of properties
/// by iteration over declared set of components in the world. The encoder might also
/// use additional resources from the world.
///
/// Every encoder must push exactly one value per iterated entity to the buffer.
pub trait StreamEncoder<'a> {
    /// List of shader properties that this encoder encodes
    type Properties: EncProperties;

    /// Get a runtime list of shader properties encoded by this encoder
    fn get_props() -> <Self::Properties as EncProperties>::PropsIter {
        Self::Properties::get_props()
    }

    /// Do the encoding, filling the provided buffer with encoded data.
    ///
    /// Unsafe because caller must guarantee that the bitset count
    /// matches the buffer length.
    ///
    /// Implementer must ensure that for every bitset entry,
    /// there is exactly one `buffer.push` call.
    unsafe fn encode<'b>(
        bitset: &BitSet,
        res: &'a Resources,
        buffer_builder: &EncodeBufferBuilder<'b>,
    );
}

pub struct LoopResult(());

pub trait EncodeLoop<'a, 'j, I, O>
where
    I: EncodingDef,
    O: EncProperties,
    Self: Sized,
{
    fn run<F>(self, mapper: F) -> LoopResult
    where
        F: Fn(
            <<I as EncodingData<'a>>::FetchedData as FetchedData<'j>>::Ref,
        ) -> <O::EncodedType as EncodingValue>::OptValue;
}

pub struct EncodeLoopImpl<'a, 'j, 'b, I, O, B>
where
    I: EncodingDef + 'a,
    O: EncProperties,
    B: EncodeBuffer<O::EncodedType>,
{
    marker: PhantomData<(I, O)>,
    bitset: &'b BitSet,
    input_data: &'j <I as EncodingData<'a>>::SystemData,
    buffer: B,
}

impl<'a, 'j, 'b, I, O, B> EncodeLoopImpl<'a, 'j, 'b, I, O, B>
where
    I: EncodingDef,
    O: EncProperties,
    B: EncodeBuffer<O::EncodedType>,
{
    fn new(
        bitset: &'b BitSet,
        input_data: &'j <I as EncodingData<'a>>::SystemData,
        buffer: B,
    ) -> Self {
        Self {
            marker: PhantomData,
            bitset,
            input_data,
            buffer,
        }
    }
}

impl<'a: 'j, 'j, 'b, I, O, B> EncodeLoop<'a, 'j, I, O> for EncodeLoopImpl<'a, 'j, 'b, I, O, B>
where
    I: EncodingDef,
    O: EncProperties,
    B: EncodeBuffer<O::EncodedType>,
{
    fn run<F>(mut self, mapper: F) -> LoopResult
    where
        F: Fn(
            <<I as EncodingData<'a>>::FetchedData as FetchedData<'j>>::Ref,
        ) -> <O::EncodedType as EncodingValue>::OptValue,
    {
        for idx in self.bitset {
            let components = <I as EncodingDef>::get_data(self.input_data, idx);
            self.buffer.push(O::resolve(mapper(components)));
        }

        LoopResult(())
    }
}

pub trait LoopingStreamEncoder<'a> {
    type Properties: EncProperties;
    type Components: EncodingDef + 'a;
    type SystemData: SystemData<'a>;

    fn encode<'j>(
        encode_loop: impl EncodeLoop<'a, 'j, Self::Components, Self::Properties>,
        system_data: Self::SystemData,
    ) -> LoopResult;
}

impl<'a, T: LoopingStreamEncoder<'a>> StreamEncoder<'a> for T {
    type Properties = T::Properties;

    unsafe fn encode<'b>(
        bitset: &BitSet,
        res: &'a Resources,
        buffer_builder: &EncodeBufferBuilder<'b>,
    ) {
        let buffer = buffer_builder.build::<T::Properties>();
        let (input_data, system_data) = SystemData::fetch(res);
        let encode_loop =
            EncodeLoopImpl::<T::Components, T::Properties, _>::new(bitset, &input_data, buffer);
        T::encode(encode_loop, system_data);
    }
}

/// A type used as an encoder output
pub type EncType<'a, T> = <<T as StreamEncoder<'a>>::Properties as EncProperties>::EncodedType;

struct AnyEncoderImpl<T> {
    _marker: std::marker::PhantomData<T>,
}

unsafe impl<T: for<'a> StreamEncoder<'a>> Send for AnyEncoderImpl<T> {}
unsafe impl<T: for<'a> StreamEncoder<'a>> Sync for AnyEncoderImpl<T> {}

/// Dynamic type that can hold any encoder
pub trait AnyEncoder: Any + Send + Sync {
    /// Get a runtime list of shader properties encoded by this encoder
    // fn get_props(&self) -> Vec<EncodedProp>;

    /// Tries to match this encoder agains a set of properties that need to be encoded.
    /// If the encoder was matched, the passed list is modified by removing the passed
    /// properties.
    ///
    /// Returns if the match was successful.
    fn try_match_props(&self, props: &mut Vec<EncodedProp>) -> bool;

    /// Run encoding operation of type-erased encoder
    ///
    /// Unsafe because caller must guarantee that the bitset count
    /// matches the buffer length.
    unsafe fn encode<'b>(
        &self,
        bitset: &BitSet,
        res: &Resources,
        buffer_builder: &EncodeBufferBuilder<'b>,
    );
}

impl<T: for<'a> StreamEncoder<'a> + 'static> AnyEncoder for AnyEncoderImpl<T> {
    fn try_match_props(&self, encoded_props: &mut Vec<EncodedProp>) -> bool {
        let is_match = T::get_props().all(|prop| {
            encoded_props
                .iter()
                .find(|&&enc_prop| prop == enc_prop)
                .is_some()
        });

        if is_match {
            // TODO: get rid of this unfortunate allocation.
            // Cannot swap_remove items from the vec while iterating over it.
            let mut indices = T::get_props()
                .map(|prop| {
                    encoded_props
                        .iter()
                        .position(|&enc_prop| prop == enc_prop)
                        .unwrap()
                })
                .collect::<Vec<_>>();

            // Indices must be removed from largest to smallest,
            // so the swaps are not going to end up replacing
            // element that should be removed in next iterations.
            indices.sort();
            for index in indices.into_iter().rev() {
                encoded_props.swap_remove(index);
            }
        }

        is_match
    }

    unsafe fn encode<'b>(
        &self,
        bitset: &BitSet,
        res: &Resources,
        buffer_builder: &EncodeBufferBuilder<'b>,
    ) {
        T::encode(bitset, res, buffer_builder);
    }
}

pub fn into_any<T: for<'a> StreamEncoder<'a> + 'static>() -> impl AnyEncoder {
    AnyEncoderImpl::<T> {
        _marker: std::marker::PhantomData,
    }
}
