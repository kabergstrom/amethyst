use super::{
    pipeline::EncoderPipeline,
    stream_encoder::{AnyEncoder, StreamEncoder},
    PipelineResolver,
};
use amethyst_core::specs::Component;
use shred::Resources;
use std::{marker::PhantomData, rc::Rc};

pub struct EncoderStorage {
    available_encoders: Vec<Rc<dyn AnyEncoder>>,
}

#[derive(Default)]
pub struct EncoderStorageBuilder {
    encoders: Vec<Rc<dyn AnyEncoder>>,
}

impl EncoderStorageBuilder {
    pub fn with_encoder<E: for<'a> StreamEncoder<'a> + 'static>(mut self) -> Self {
        use super::stream_encoder::into_any;
        self.encoders.push(Rc::new(into_any::<E>()));
        self
    }
    pub fn build(self) -> EncoderStorage {
        EncoderStorage {
            available_encoders: self.encoders,
        }
    }
}

impl EncoderStorage {
    fn encoders_for_pipeline(&self, pipeline: &EncoderPipeline) -> Vec<Rc<dyn AnyEncoder>> {
        unimplemented!()
    }
}

/// Defines a query to the encoding system.
///
/// Every query has one “central” component `T` that must be present on entities of interest.
/// This allows to avoid unintentional multiple renders by many passes.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct EncodingQuery<T, R>
where
    T: Component,
    R: PipelineResolver<T>,
{
    _marker: PhantomData<T>,
    pipeline_resolver: R,
}

impl<T, R> EncodingQuery<T, R>
where
    T: Component,
    R: PipelineResolver<T>,
{
    pub fn new(pipeline_resolver: R) -> Self {
        EncodingQuery {
            _marker: PhantomData,
            pipeline_resolver,
        }
    }

    pub fn execute(&self, res: &Resources) {}
}

#[derive(Default)]
pub struct EncodingSet {
    encoders: Vec<Rc<dyn AnyEncoder>>,
}
