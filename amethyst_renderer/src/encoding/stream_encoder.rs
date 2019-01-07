use crate::encoding::attributes::EncodedProp;
use crate::encoding::{EncAttributes, EncodeBuffer, EncodingSet};
use amethyst_core::specs::{join::JoinIter, SystemData};
use core::any::Any;

/// A main trait that defines a strategy to encode specified stream of attributes
/// by iteration over declared set of components in the world. The encoder might also
/// use additional resources from the world.
///
/// Every encoder must push exactly one value per iterated entity to the buffer.
pub trait StreamEncoder {
    type Attributes: EncAttributes;

    fn encode<'a: 'j, 'j, B: EncodeBuffer<EncType<'a, 'j, Self>>>(
        buffer: &mut B,
        iter: IterType<'a, 'j, Self>,
        system_data: DataType<'a, 'j, Self>,
    ) where
        Self: StreamEncoderData<'a, 'j>;
}

pub trait StreamEncoderData<'a, 'j> {
    type Components: EncodingSet<'j>;
    type SystemData: SystemData<'a>;
}

pub type EncType<'a, 'j, T> = <<T as StreamEncoder>::Attributes as EncAttributes>::EncodedType;
pub type IterType<'a, 'j, T> =
    JoinIter<<<T as StreamEncoderData<'a, 'j>>::Components as EncodingSet<'j>>::Joined>;
pub type DataType<'a, 'j, T> = <T as StreamEncoderData<'a, 'j>>::SystemData;

fn encoder_encode<'a: 'j, 'j, T, B>(
    buffer: &mut B,
    iter: IterType<'a, 'j, T>,
    system_data: DataType<'a, 'j, T>,
) where
    T: StreamEncoder + StreamEncoderData<'a, 'j>,
    B: EncodeBuffer<EncType<'a, 'j, T>>,
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
        <T::Attributes as EncAttributes>::get_props()
    }
}

pub fn into_any<T: StreamEncoder + 'static>() -> impl AnyEncoder {
    AnyEncoderImpl::<T> {
        _marker: std::marker::PhantomData,
    }
}
