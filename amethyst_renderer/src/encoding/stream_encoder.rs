use crate::encoding::{
    data::EncodingDef, properties::EncodedProp, EncProperties, EncodeBuffer, EncodingSet,
};
use amethyst_core::specs::SystemData;
use core::any::Any;

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
        buffer: &mut impl EncodeBuffer<EncType<'a, 'j, Self>>,
        iter: impl Iterator<Item = IterItem<'a, 'j, Self>>,
        system_data: DataType<'a, Self>,
    );
}

pub type EncType<'a, 'j, T> = <<T as StreamEncoder<'a>>::Properties as EncProperties>::EncodedType;
pub type IterItem<'a, 'j, T> = <<T as StreamEncoder<'a>>::Components as EncodingSet<'j>>::IterItem;
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
    fn get_count<'a>(&self, res: &'a shred::Resources) -> usize;
}

impl<T: for<'a> StreamEncoder<'a> + 'static> AnyEncoder for AnyEncoderImpl<T> {
    fn get_props(&self) -> Vec<EncodedProp> {
        T::get_props()
    }

    fn get_count<'a>(&self, res: &'a shred::Resources) -> usize {
        let data = T::Components::fetch(res);
        // this fails
        // let joinable = T::Components::joinable(&data);

        // the trait bound `<T as StreamEncoder<'_>>::Components: EncodingJoin<'_, '_>` is not satisfied
        // the trait `EncodingJoin<'_, '_>` is not implemented for `<T as StreamEncoder<'_>>::Components`
        // help: consider adding a `where <T as StreamEncoder<'_>>::Components: EncodingJoin<'_, '_>` bound

        unimplemented!();
    }
}

pub fn into_any<T: for<'a> StreamEncoder<'a> + 'static>() -> impl AnyEncoder {
    AnyEncoderImpl::<T> {
        _marker: std::marker::PhantomData,
    }
}
