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
    unsafe fn encode<B: EncodeBuffer<EncType<'a, Self>>>(
        bitset: &BitSet,
        res: &'a Resources,
        buffer: B,
    );
}

pub struct LoopResult(());

pub trait EncodeLoop<I, O>
where
    I: EncodingDef,
    O: EncProperties,
    Self: Sized,
{
    fn run<F>(self, mapper: F) -> LoopResult
    where
        for<'a, 'j> F: Fn(
            <<I as EncodingData<'a>>::FetchedData as FetchedData<'j>>::Ref,
        ) -> <O::EncodedType as EncodingValue>::OptValue;
}

pub struct EncodeLoopImpl<'a, 'b, I, O, B>
where
    I: EncodingDef + 'a,
    O: EncProperties,
    B: EncodeBuffer<O::EncodedType>,
{
    marker: PhantomData<(I, O)>,
    bitset: &'a BitSet,
    input_data: <I as EncodingData<'b>>::SystemData,
    buffer: B,
}

impl<'a, 'b, I, O, B> EncodeLoopImpl<'a, 'b, I, O, B>
where
    I: EncodingDef,
    O: EncProperties,
    B: EncodeBuffer<O::EncodedType>,
{
    fn new(bitset: &'a BitSet, input_data: <I as EncodingData<'b>>::SystemData, buffer: B) -> Self {
        Self {
            marker: PhantomData,
            bitset,
            input_data,
            buffer,
        }
    }
}

impl<I, O, B> EncodeLoop<I, O> for EncodeLoopImpl<'_, '_, I, O, B>
where
    I: EncodingDef,
    O: EncProperties,
    B: EncodeBuffer<O::EncodedType>,
{
    fn run<F>(mut self, mapper: F) -> LoopResult
    where
        for<'a, 'j> F: Fn(
            <<I as EncodingData<'a>>::FetchedData as FetchedData<'j>>::Ref,
        ) -> <O::EncodedType as EncodingValue>::OptValue,
    {
        for idx in self.bitset {
            let components = <I as EncodingDef>::get_data(&self.input_data, idx);
            self.buffer.push(O::resolve(mapper(components)));
        }

        LoopResult(())
    }
}

pub trait LoopingStreamEncoder<'a> {
    type Properties: EncProperties;
    type Components: EncodingDef;
    type SystemData: SystemData<'a>;

    fn encode(
        encode_loop: impl EncodeLoop<Self::Components, Self::Properties>,
        system_data: Self::SystemData,
    ) -> LoopResult;
}

impl<'a, T: LoopingStreamEncoder<'a>> StreamEncoder<'a> for T {
    type Properties = T::Properties;

    unsafe fn encode<B: EncodeBuffer<EncType<'a, Self>>>(
        bitset: &BitSet,
        res: &'a Resources,
        buffer: B,
    ) {
        let (input_data, system_data) = SystemData::fetch(res);
        let encode_loop =
            EncodeLoopImpl::<T::Components, T::Properties, B>::new(bitset, input_data, buffer);
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
    fn get_props(&self) -> Vec<EncodedProp>;

    /// Run encoding operation of type-erased encoder
    unsafe fn encode<'b>(
        &self,
        bitset: &BitSet,
        res: &Resources,
        buffer_builder: EncodeBufferBuilder<'b>,
    );
}

impl<T: for<'a> StreamEncoder<'a> + 'static> AnyEncoder for AnyEncoderImpl<T> {
    fn get_props(&self) -> Vec<EncodedProp> {
        T::get_props().collect()
    }
    unsafe fn encode<'b>(
        &self,
        bitset: &BitSet,
        res: &Resources,
        buffer_builder: EncodeBufferBuilder<'b>,
    ) {
        T::encode(bitset, res, buffer_builder.build());
    }
}

pub fn into_any<T: for<'a> StreamEncoder<'a> + 'static>() -> impl AnyEncoder {
    AnyEncoderImpl::<T> {
        _marker: std::marker::PhantomData,
    }
}
