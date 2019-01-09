use crate::encoding::properties::EncodedProp;
use crate::encoding::{EncProperties, EncodeBuffer, EncodingSet};
use amethyst_core::specs::{join::Join, SystemData};
use core::any::Any;

/// A main trait that defines a strategy to encode specified stream of properties
/// by iteration over declared set of components in the world. The encoder might also
/// use additional resources from the world.
///
/// Every encoder must push exactly one value per iterated entity to the buffer.
pub trait StreamEncoder {
    type Properties: EncProperties;

    fn encode<'a: 'j, 'j>(
        buffer: &mut impl EncodeBuffer<EncType<'a, 'j, Self>>,
        iter: impl Iterator<Item = IterItem<'a, 'j, Self>>,
        storage: DataType<'a, 'j, Self>,
    ) where
        Self: StreamEncoderData<'a, 'j>;
}

pub trait StreamEncoderData<'a, 'j> {
    type Components: EncodingSet<'j>;
    type SystemData: SystemData<'a>;
}

pub type EncType<'a, 'j, T> = <<T as StreamEncoder>::Properties as EncProperties>::EncodedType;
pub type IterItem<'a, 'j, T> =
    <<<T as StreamEncoderData<'a, 'j>>::Components as EncodingSet<'j>>::Joined as Join>::Type;
pub type DataType<'a, 'j, T> = <T as StreamEncoderData<'a, 'j>>::SystemData;

fn encoder_encode<'a: 'j, 'j, T>(
    buffer: &mut impl EncodeBuffer<EncType<'a, 'j, T>>,
    iter: impl Iterator<Item = IterItem<'a, 'j, T>>,
    system_data: DataType<'a, 'j, T>,
) where
    T: StreamEncoder + StreamEncoderData<'a, 'j>,
{
    T::encode(buffer, iter, system_data)
}

struct AnyEncoderImpl<T> {
    _marker: std::marker::PhantomData<T>,
}

unsafe impl<T: StreamEncoder> Send for AnyEncoderImpl<T> {}
unsafe impl<T: StreamEncoder> Sync for AnyEncoderImpl<T> {}

pub trait AnyEncoder: Any + Send + Sync {
    fn get_encoder_props(&self) -> Vec<EncodedProp>;
}

impl<T> AnyEncoder for AnyEncoderImpl<T>
where
    T: StreamEncoder + 'static,
{
    fn get_encoder_props(&self) -> Vec<EncodedProp> {
        <T::Properties as EncProperties>::get_props()
    }
}

pub fn into_any<T: StreamEncoder + 'static>() -> impl AnyEncoder {
    AnyEncoderImpl::<T> {
        _marker: std::marker::PhantomData,
    }
}
