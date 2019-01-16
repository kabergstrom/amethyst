use super::{EncoderPipeline, Shader};
use amethyst_assets::{Handle, Loader};
use amethyst_core::specs::{storage::UnprotectedStorage, Component, VecStorage, Write};
use hibitset::BitSet;
use shred::Resources;

/// Ability to resolve pipeline based on a component. Used in the first phase of world encoding.
pub trait PipelineResolver<T: Component> {
    /// Resolve a pipeline from the world based on a component information
    fn resolve(&self, res: &Resources, component: &T) -> Option<Handle<EncoderPipeline>>;
}

/// A simplified version of PipelineResolver. Provides a way to specify the resolution
/// by just extracting the shader handle from a component.
/// Usually used in it's closure form `Fn(&T) -> Handle<Shader>`.
///
/// Note that further down, resolved pipelines are cached
/// based on handles returned from this trait's `resolve` method.
pub trait ShaderResolver<T: Component> {
    /// The shader resolution method that retreives a shader handle from a component.
    fn resolve(&self, res: &Resources, component: &T) -> Handle<Shader>;
}

impl<T: Component, R: ShaderResolver<T>> PipelineResolver<T> for R {
    fn resolve(&self, res: &Resources, component: &T) -> Option<Handle<EncoderPipeline>> {
        let shader_handle = ShaderResolver::resolve(self, res, component);
        res.fetch_mut::<Write<'_, PipelineResolveCache>>()
            .resolve(res, shader_handle)
    }
}

type HandleVersion = usize;

#[derive(Default)]
struct PipelineResolveCache(BitSet, VecStorage<(HandleVersion, Handle<EncoderPipeline>)>);
impl PipelineResolveCache {
    fn resolve(
        &mut self,
        res: &Resources,
        shader_handle: Handle<Shader>,
    ) -> Option<Handle<EncoderPipeline>> {
        let id = shader_handle.id();
        let shader_version: HandleVersion = 0; // TODO: read real version once hot-reloading is done

        if self.0.contains(id) {
            let (version, handle) = unsafe { self.1.get(id) };
            if *version == shader_version {
                return Some(handle.clone());
            } else {
                unsafe { self.1.remove(id) };
                self.0.remove(id);
            }
        }

        EncoderPipeline::from_shader(res, &shader_handle).map(|pipeline| {
            let loader = res.fetch_mut::<Loader>();
            let handle = loader.load_from_data(pipeline, (), &res.fetch());

            unsafe { self.1.insert(id, (shader_version, handle.clone())) };
            self.0.add(id);
            handle
        })
    }
}

impl<T, R> ShaderResolver<T> for R
where
    T: Component,
    R: Fn(&T) -> Handle<Shader>,
{
    fn resolve(&self, _res: &Resources, component: &T) -> Handle<Shader> {
        self(component)
    }
}
