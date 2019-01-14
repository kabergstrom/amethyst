use super::{
    bitset::VecBitSet, data::EncodingDef, properties::EncodingValue, EncProperties, EncodeBuffer,
    EncodedProp, EncodingData, FetchedData,
};
use amethyst_core::specs::SystemData;
use core::any::Any;
use core::marker::PhantomData;
use hibitset::BitSet;
use shred::Resources;

/// A main trait that defines a strategy to encode specified stream of properties
/// by iteration over declared set of components in the world. The encoder might also
/// use additional resources from the world.
///
/// Every encoder must push exactly one value per iterated entity to the buffer.
pub trait StreamEncoder<'a> {
    type Properties: EncProperties;
    type Components: EncodingDef;
    type SystemData: SystemData<'a>;

    fn get_props() -> Vec<EncodedProp> {
        Self::Properties::get_props()
    }

    fn encode<'j>(
        buffer: &mut impl EncodeBuffer<EncType<'a, Self>>,
        iter: impl Iterator<Item = IterItem<'a, 'j, Self>>,
        system_data: DataType<'a, Self>,
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

pub struct EncodeLoopImpl<'a, I, O, B>
where
    I: EncodingDef + 'a,
    O: EncProperties,
    B: EncodeBuffer<O::EncodedType>,
{
    marker: PhantomData<(I, O)>,
    bitset: &'a BitSet,
    res: &'a Resources,
    buffer: &'a mut B,
}

impl<'a, I, O, B> EncodeLoopImpl<'a, I, O, B>
where
    I: EncodingDef + 'a,
    O: EncProperties,
    B: EncodeBuffer<O::EncodedType>,
{
    fn new(bitset: &'a BitSet, res: &'a Resources, buffer: &'a mut B) -> Self {
        Self {
            marker: PhantomData,
            bitset,
            res,
            buffer,
        }
    }
}

impl<'x, I, O, B> EncodeLoop<I, O> for EncodeLoopImpl<'x, I, O, B>
where
    I: EncodingDef + 'x,
    O: EncProperties,
    B: EncodeBuffer<O::EncodedType>,
{
    fn run<F>(self, mapper: F) -> LoopResult
    where
        for<'a, 'j> F: Fn(
            <<I as EncodingData<'a>>::FetchedData as FetchedData<'j>>::Ref,
        ) -> <O::EncodedType as EncodingValue>::OptValue,
    {
        let data = I::fetch(self.res);

        for idx in self.bitset {
            let components = <I as EncodingDef>::get_data(&data, idx);
            let encoded = mapper(components);
            let resolved = O::resolve(encoded);
            self.buffer.push(resolved);
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
    type Components = T::Components;
    type SystemData = T::SystemData;

    fn encode<'j>(
        buffer: &mut impl EncodeBuffer<EncType<'a, Self>>,
        iter: impl Iterator<Item = IterItem<'a, 'j, Self>>,
        system_data: DataType<'a, Self>,
    ) {
        // let looping = EncodeLoopImpl::new();
    }
}

pub type EncType<'a, T> = <<T as StreamEncoder<'a>>::Properties as EncProperties>::EncodedType;
pub type IterItem<'a, 'j, T> = <<<T as StreamEncoder<'a>>::Components as EncodingData<'a>>::FetchedData as FetchedData<'j>>::Ref;
pub type DataType<'a, T> = <T as StreamEncoder<'a>>::SystemData;

// fn encoder_encode<'a: 'j, 'j, T>(
//     buffer: &mut impl EncodeBuffer<EncType<'a, 'j, T>>,
//     iter: impl Iterator<Item = IterItem<'a, 'j, T>>,
//     system_data: DataType<'a, 'j, T>,
// ) where
//     T: StreamEncoder<'a>,
//     <T as StreamEncoder<'a>>::Components: EncodingSet<'j>,
// {
//     T::encode(buffer, iter, system_data)
// }

struct AnyEncoderImpl<T> {
    _marker: std::marker::PhantomData<T>,
}

unsafe impl<T: for<'a> StreamEncoder<'a>> Send for AnyEncoderImpl<T> {}
unsafe impl<T: for<'a> StreamEncoder<'a>> Sync for AnyEncoderImpl<T> {}

pub trait AnyEncoder: Any + Send + Sync {
    fn get_props(&self) -> Vec<EncodedProp>;
    fn get_masks<'a>(&self, res: &'a Resources) -> VecBitSet<'a>;
}

impl<T: for<'a> StreamEncoder<'a> + 'static> AnyEncoder for AnyEncoderImpl<T> {
    fn get_props(&self) -> Vec<EncodedProp> {
        T::get_props()
    }

    fn get_masks<'a>(&self, res: &'a Resources) -> VecBitSet<'a> {
        let data = T::Components::fetch(res);
        // T::Components::get_masks(&data)
        unimplemented!();
    }
}

pub fn into_any<T: for<'a> StreamEncoder<'a> + 'static>() -> impl AnyEncoder {
    AnyEncoderImpl::<T> {
        _marker: std::marker::PhantomData,
    }
}
