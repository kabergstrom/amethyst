use super::properties::EncodedProp;
use amethyst_assets::{Asset, Handle, Loader};
use amethyst_core::specs::{storage::UnprotectedStorage, Component, VecStorage, Write};
use fnv::FnvHashMap;
use hibitset::BitSet;
use shred::Resources;
use std::marker::PhantomData;

// TODO: use actual shaders
struct Shader;

/// A set of properties used for encoding.
/// Derived from a shader.
struct EncoderPipeline {
    properties: Vec<EncodedProp>,
}
impl Asset for EncoderPipeline {
    const NAME: &'static str = "EncoderPipeline";
    type HandleStorage = VecStorage<Handle<Self>>;
    type Data = EncoderPipeline;
}

impl EncoderPipeline {
    // TODO: the input will probably need more than a handle, but that's ok for prototyping
    fn from_shader(res: &Resources, shader: &Handle<Shader>) -> Self {
        EncoderPipeline { properties: vec![] }
    }
}

trait PipelineResolver<'a, T: Component> {
    fn resolve(&self, res: &'a Resources, component: &T) -> Handle<EncoderPipeline>;
}

trait ShaderResolver<'a, T: Component> {
    fn resolve(&self, res: &'a Resources, component: &T) -> Handle<Shader>;
}

impl<'a, T: Component, R: ShaderResolver<'a, T>> PipelineResolver<'a, T> for R {
    fn resolve(&self, res: &'a Resources, component: &T) -> Handle<EncoderPipeline> {
        let shader = ShaderResolver::resolve(self, res, component);
        res.fetch_mut::<Write<'_, PipelineResolveCache>>()
            .resolve(res, shader)
    }
}

type HandleVersion = usize;
struct PipelineResolveCache(BitSet, VecStorage<(HandleVersion, Handle<EncoderPipeline>)>);
impl PipelineResolveCache {
    fn resolve(&mut self, res: &Resources, shader: Handle<Shader>) -> Handle<EncoderPipeline> {
        let id = shader.id();
        let shader_version: HandleVersion = 0; // TODO: read real version once hot-reloading is done

        if self.0.contains(id) {
            let (version, handle) = unsafe { self.1.get(id) };
            if *version == shader_version {
                return handle.clone();
            } else {
                unsafe { self.1.remove(id) };
                self.0.remove(id);
            }
        }

        let pipeline = EncoderPipeline::from_shader(res, &shader);
        let loader = res.fetch_mut::<Loader>();
        let handle = loader.load_from_data(pipeline, (), &res.fetch());

        unsafe { self.1.insert(id, (shader_version, handle.clone())) };
        self.0.add(id);

        handle
    }
}

impl<'a, T, R> ShaderResolver<'a, T> for R
where
    T: Component,
    R: Fn(&T) -> Handle<Shader>,
{
    fn resolve(&self, res: &'a Resources, component: &T) -> Handle<Shader> {
        self(component)
    }
}

/// every query has one “central” component `T` that must be present on entities of interest.
/// This allows to avoid unintentional multiple renders by many passes.
struct EncodingQuery<'a, T, R>
where
    T: Component,
    R: PipelineResolver<'a, T>,
{
    _marker: PhantomData<&'a T>,
    pipeline_resolver: R,
    // TODO: support debug fallback rendering
    // fallbacks: Vec<(Handle<Shader>, Encoder)>
}

impl<'a, T, R> EncodingQuery<'a, T, R>
where
    T: Component,
    R: PipelineResolver<'a, T>,
{
    fn new(pipeline_resolver: R) -> Self {
        EncodingQuery {
            _marker: PhantomData,
            pipeline_resolver,
        }
    }
}

// "test" code

struct TestCentralComponent(Handle<Shader>);
impl Component for TestCentralComponent {
    type Storage = VecStorage<Self>;
}

fn test_method(_res: &Resources) {
    let query = EncodingQuery::new(|c: &TestCentralComponent| c.0.clone());
}
