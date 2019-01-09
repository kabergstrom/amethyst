mod buffer;
mod data;
mod encoders_impl;
mod properties;
mod properties_impl;
mod stream_encoder;

#[cfg(test)]
mod test;

pub use self::{
    buffer::EncodeBuffer,
    data::{DeferredEncodingSet, Encode, EncodingSet, MaybeEncode},
    properties::{EncProperties, EncProperty, EncodedProp, ShaderInput, ShaderInputType},
    stream_encoder::{AnyEncoder, DataType, EncType, IterItem, StreamEncoder, StreamEncoderData},
};
use core::hash::Hash;
use fnv::{FnvHashMap, FnvHashSet};
use std::rc::Rc;

// This file now contains mostly in-progress things that aren't yet put into a separate module.
// Just playing with ideas here.

/// A list of encoders that have to run (possibly in parallel) in order to encode
/// the entire set of required shader properties (shader layout).
pub struct EncoderBundle {
    encoders: Vec<Rc<dyn AnyEncoder>>,
}

impl EncoderBundle {
    fn new(encoders: Vec<Rc<dyn AnyEncoder>>) -> Self {
        Self { encoders }
    }

    fn count(&self, res: &shred::Resources) -> usize {
        0
    }
}

/// A list of encoders that have to run (usually in sequence) in order to encode
/// all possible component permutations for a given shader layout.
pub struct LayoutEncoder {
    bundles: Vec<EncoderBundle>,
}

impl LayoutEncoder {
    fn new(bundles: Vec<EncoderBundle>) -> Self {
        Self { bundles }
    }
    fn empty() -> Self {
        Self { bundles: vec![] }
    }

    pub fn count(&self, res: &shred::Resources) -> usize {
        self.bundles.iter().map(|b| b.count(res)).sum()
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct EncoderQuery {
    props: Vec<EncodedProp>,
}

impl EncoderQuery {
    // TODO: spirv-reflect support
    // pub fn from_shader_introspection() {

    // }

    pub fn new(props: Vec<EncodedProp>) -> Self {
        Self { props }
    }
}

// a resource for querying and holding registrations
pub struct WorldEncoder {
    // encoders_by_props: FnvHashMap<EncodedProp, Vec<Rc<dyn AnyEncoder>>>,
    available_encoders: Vec<Rc<dyn AnyEncoder>>,
    layout_cache: FnvHashMap<EncoderQuery, LayoutEncoder>,
}

#[derive(Debug)]
pub struct EncodingError;

#[allow(dead_code)]
impl WorldEncoder {
    pub fn build() -> EncodingBuilder {
        EncodingBuilder::default()
    }

    /// Retreive a cached LayoutEncoder for specified Query
    pub fn get_query_encoder<'a>(&'a mut self, query: &EncoderQuery) -> &'a LayoutEncoder {
        let is_in_cache = self.layout_cache.contains_key(query);

        if !is_in_cache {
            let encoder = self.calculate_query_encoder(query);
            self.layout_cache.insert(query.clone(), encoder);
        }

        self.layout_cache.get(query).unwrap()
    }

    fn calculate_query_encoder(&self, query: &EncoderQuery) -> LayoutEncoder {
        // find suitable encoder bundles
        let back_map = self
            .available_encoders
            .iter()
            .map(|enc| (enc.get_encoder_props(), enc))
            .collect::<FnvHashMap<_, _>>();

        let prop_groups = back_map.keys().collect::<Vec<_>>();

        let props_set = query.props.iter().cloned().collect::<FnvHashSet<_>>();

        // TODO: support multiple solutions
        let solution = greedy_set_cover(prop_groups, props_set);

        if let Ok(matched) = solution {
            let encoders = matched
                .iter()
                .map(|&list| (*back_map.get(list).unwrap()).clone())
                .collect::<Vec<_>>();
            LayoutEncoder::new(vec![EncoderBundle::new(encoders)])
        } else {
            LayoutEncoder::empty()
        }
    }

    /// Retreive the exact size of byte buffer that has to be allocated
    pub fn query_size_hint(&mut self, res: &shred::Resources, query: &EncoderQuery) -> usize {
        self.get_query_encoder(query).count(res)
    }

    /// Execute query for given layout of shader input data
    pub fn query(
        &mut self,
        res: &shred::Resources,
        query: &EncoderQuery,
        buffer: &mut [u8],
    ) -> Result<(), EncodingError> {
        // TODO: Return AttributeBindingIterator instead of ()

        let enc = self.get_query_encoder(query);
        Ok(())
    }
}

fn greedy_set_cover<K: Eq + Hash>(
    mut groups: Vec<&Vec<K>>,
    mut keys_to_cover: FnvHashSet<K>,
) -> Result<Vec<&Vec<K>>, ()> {
    let mut result = Vec::new();

    while !keys_to_cover.is_empty() {
        purge_out_of_set(&mut groups, &keys_to_cover);

        if let Some((i, _)) = groups
            .iter()
            .enumerate()
            .map(|(i, group)| (i, group.len()))
            .max_by_key(|&(_, len)| len)
        {
            let best = groups.remove(i);
            for k in best {
                keys_to_cover.remove(k);
            }
            result.push(best);
        } else {
            return Err(()); // no greedy solution
        }
    }

    Ok(result)
}

fn purge_out_of_set<K: Eq + Hash>(groups: &mut Vec<&Vec<K>>, keys_to_cover: &FnvHashSet<K>) {
    let out_of_set = groups.iter().enumerate().filter_map(|(i, group)| {
        if group.iter().all(|k| keys_to_cover.contains(k)) {
            None
        } else {
            Some(i)
        }
    });

    for group_idx in out_of_set.collect::<Vec<_>>() {
        groups.remove(group_idx);
    }
}

fn vecmap_insert<K: Eq + Hash, V>(map: &mut FnvHashMap<K, Vec<V>>, key: K, value: V) {
    if let Some(vec) = map.get_mut(&key) {
        vec.push(value);
    } else {
        map.insert(key, vec![value]);
    }
}

#[derive(Default)]
pub struct EncodingBuilder {
    encoders: Vec<Rc<dyn AnyEncoder>>,
}

impl EncodingBuilder {
    // registration might be totally compile-time
    pub fn with_encoder<E: StreamEncoder + 'static>(mut self) -> Self {
        use self::stream_encoder::into_any;
        self.encoders.push(Rc::new(into_any::<E>()));
        self
    }
    pub fn build(self) -> WorldEncoder {
        // let mut encoders_by_props = FnvHashMap::default();

        // for encoder in self.encoders {
        //     for prop in encoder.get_encoder_props() {
        //         vecmap_insert(&mut encoders_by_props, prop, encoder.clone());
        //     }
        // }

        WorldEncoder {
            available_encoders: self.encoders,
            layout_cache: Default::default(),
        }
    }
}
