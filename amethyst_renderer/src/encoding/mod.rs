mod attributes;
mod attributes_impl;
mod buffer;
mod data;
mod encoders_impl;
mod stream_encoder;

pub use self::{
    attributes::{EncAttribute, EncAttributes, ShaderInputType},
    buffer::EncodeBuffer,
    data::{Encode, EncodingSet},
    stream_encoder::{EncType, IterType, StreamEncoder},
};
use shred::SystemData;
use std::marker::PhantomData;

// This file now contains mostly in-progress things that aren't yet put into a separate module.
// Just playing with ideas here.

/// A list of encoders that have to run (possibly in parallel) in order to encode
/// the entire set of required shader attributes (shader layout).
trait EncoderBundle<'a, 'j> {
    type Attributes: EncAttributes;
    type Components: EncodingSet<'j>;
    type SystemData: SystemData<'a>;

    fn encode_bundle();
}

impl<'a, 'j, A, B> EncoderBundle<'a, 'j> for (A, B)
where
    A: StreamEncoder<'a, 'j>,
    B: StreamEncoder<'a, 'j>,
{
    type Attributes = (A::Attributes, B::Attributes);
    type Components = (A::Components, B::Components);
    type SystemData = (A::SystemData, B::SystemData);

    fn encode_bundle() {
        // todo: externally visible attributes should be flattened for querying
        // (possibly at runtime? but then joining encoder bundles will possibly be harder)

        // todo: prepare interleaved buffer view from flat buf
    }
}

/// A list of encoders that have to run (usually in sequence) in order to encode
/// all possible component permutations for a given shader layout.
struct LayoutEncoder {} // TODO

/// Unit of work for encoding.
struct EncodingUnit {}

struct EncoderLayout {
    stride: usize,
}

// TODO: layouts should probably support

trait StaticLayout {
    type ReturnType;
}
struct Layout {}

struct EncoderQuery<L: StaticLayout> {
    dynamic_layout: Layout,
    phantom: PhantomData<L>,
}

// a resource for querying and holding registrations
struct WorldEncoder {
    // available_encoders: HashMap<EncodedTypeSet, StreamEncoder>,
}

struct EncodingError;

impl WorldEncoder {
    fn build() -> EncodingBuilder {
        EncodingBuilder::default()
    }

    /// Retreive the exact size of byte buffer that has to be allocated
    fn query_size_hint<L: StaticLayout>(&self) -> usize {
        unimplemented!();
    }

    fn query<L: StaticLayout>(
        &self,
        query: EncoderQuery<L>,
        buffer: &mut [u8],
    ) -> Result<L::ReturnType, EncodingError> {
        unimplemented!();
    }
}

#[derive(Default)]
struct EncodingBuilder;
impl EncodingBuilder {
    // registration might be totally compile-time
    fn with_encoder<E: for<'a, 'j> StreamEncoder<'a, 'j>>() {
        unimplemented!()
    }
    fn build(self) -> WorldEncoder {
        unimplemented!()
    }
}
