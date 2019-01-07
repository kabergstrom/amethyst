mod attributes;
mod attributes_impl;
mod buffer;
mod data;
mod encoders_impl;
mod stream_encoder;
mod test;

pub use self::{
    attributes::{EncAttribute, EncAttributes, EncodedProp, ShaderInputType},
    buffer::EncodeBuffer,
    data::{Encode, EncodingSet},
    stream_encoder::{AnyEncoder, DataType, EncType, IterType, StreamEncoder, StreamEncoderData},
};
use core::hash::Hash;
use fnv::FnvHashMap;
use shred::SystemData;
use std::marker::PhantomData;

// This file now contains mostly in-progress things that aren't yet put into a separate module.
// Just playing with ideas here.

/// A list of encoders that have to run (possibly in parallel) in order to encode
/// the entire set of required shader attributes (shader layout).
trait EncoderBundle {} // TODO

/// A list of encoders that have to run (usually in sequence) in order to encode
/// all possible component permutations for a given shader layout.
struct LayoutEncoder {} // TODO

pub trait StaticLayout {
    type ReturnType;
}

impl StaticLayout for () {
    type ReturnType = ();
}

pub struct Layout {}

pub struct EncoderQuery<L: StaticLayout> {
    dynamic_layout: Layout,
    _marker: PhantomData<L>,
}

impl<L: StaticLayout> EncoderQuery<L> {
    pub fn new() -> Self {
        Self {
            dynamic_layout: Layout {},
            _marker: PhantomData,
        }
    }
}

// a resource for querying and holding registrations
struct WorldEncoder {
    available_encoders: FnvHashMap<EncodedProp, Vec<Box<dyn AnyEncoder>>>,
}

#[derive(Debug)]
struct EncodingError;

impl WorldEncoder {
    fn build() -> EncodingBuilder {
        EncodingBuilder::default()
    }

    /// Retreive the exact size of byte buffer that has to be allocated
    fn query_size_hint<L: StaticLayout>(
        &self,
        res: &shred::Resources,
        query: &EncoderQuery<L>,
    ) -> usize {
        unimplemented!();
    }

    fn query<L: StaticLayout>(
        &self,
        res: &shred::Resources,
        query: &EncoderQuery<L>,
        buffer: &mut [u8],
    ) -> Result<L::ReturnType, EncodingError> {
        unimplemented!();
    }
}

#[derive(Default)]
struct EncodingBuilder {
    map: FnvHashMap<EncodedProp, Vec<Box<dyn AnyEncoder>>>,
}

fn vecmap_insert<K: Eq + Hash, V>(map: &mut FnvHashMap<K, Vec<V>>, key: K, value: V) {
    if let Some(vec) = map.get_mut(&key) {
        vec.push(value);
    } else {
        map.insert(key, vec![value]);
    }
}

impl EncodingBuilder {
    // registration might be totally compile-time
    fn with_encoder<E: StreamEncoder + 'static>(mut self) -> Self {
        use self::stream_encoder::into_any;

        let enc = into_any::<E>();

        for prop in enc.get_encoder_props() {
            vecmap_insert(&mut self.map, prop, Box::new(into_any::<E>()));
        }

        self
    }
    fn build(self) -> WorldEncoder {
        WorldEncoder {
            available_encoders: self.map,
        }
    }
}
