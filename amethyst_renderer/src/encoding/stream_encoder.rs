use crate::encoding::{EncAttributes, EncodeBuffer, EncodingSet};
use amethyst_core::specs::{join::JoinIter, SystemData};

/// A main trait that defines a strategy to encode specified stream of attributes
/// by iteration over declared set of components in the world. The encoder might also
/// use additional resources from the world.
///
/// Every encoder must push exactly one value per iterated entity to the buffer.
pub trait StreamEncoder<'a, 'j> {
    type Attributes: EncAttributes;
    type Components: EncodingSet<'j>;
    type SystemData: SystemData<'a>;

    fn encode<B: EncodeBuffer<EncType<'a, 'j, Self>>>(
        buffer: &mut B,
        iter: IterType<'a, 'j, Self>,
        system_data: Self::SystemData,
    );
}

pub type EncType<'a, 'j, T> =
    <<T as StreamEncoder<'a, 'j>>::Attributes as EncAttributes>::EncodedType;
pub type IterType<'a, 'j, T> =
    JoinIter<<<T as StreamEncoder<'a, 'j>>::Components as EncodingSet<'j>>::Joined>;
