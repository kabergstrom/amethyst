use super::{EncodingLayout, Shader};
use amethyst_assets::{AssetStorage, Handle, Loader, Processor};
use amethyst_core::specs::{
    storage::UnprotectedStorage, Component, RunNow, SystemData, VecStorage, Write,
};
use hibitset::BitSet;
use shred::Resources;

/// Ability to resolve pipeline based on a component. Used in the first phase of world encoding.
pub trait LayoutResolver<T: Component> {
    /// Resolve a pipeline from the world based on a component information
    fn resolve(&self, res: &Resources, component: &T) -> Option<Handle<EncodingLayout>>;
}

/// A simplified version of LayoutResolver. Provides a way to specify the resolution
/// by just extracting the shader handle from a component.
/// Usually used in it's closure form `Fn(&T) -> Handle<Shader>`.
///
/// Note that further down, resolved pipelines are cached
/// based on handles returned from this trait's `resolve` method.
pub trait ShaderResolver<T: Component> {
    /// The shader resolution method that retreives a shader handle from a component.
    fn resolve(&self, res: &Resources, component: &T) -> Handle<Shader>;
}

impl<T: Component, R: ShaderResolver<T>> LayoutResolver<T> for R {
    fn resolve(&self, res: &Resources, component: &T) -> Option<Handle<EncodingLayout>> {
        let shader_handle = ShaderResolver::resolve(self, res, component);
        <Write<'_, LayoutResolveCache>>::fetch(res).resolve(res, shader_handle)
    }
}

type HandleVersion = usize;

/// A resource used to cache resolved layounts based on shader handles.
#[derive(Default)]
pub struct LayoutResolveCache(BitSet, VecStorage<(HandleVersion, Handle<EncodingLayout>)>);
impl LayoutResolveCache {
    fn resolve(
        &mut self,
        res: &Resources,
        shader_handle: Handle<Shader>,
    ) -> Option<Handle<EncodingLayout>> {
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

        let storage = res.fetch::<AssetStorage<Shader>>();
        let maybe_shader = storage.get(&shader_handle);
        maybe_shader.map(|shader| {
            let layout = EncodingLayout::from_shader(&shader);
            let loader = res.fetch::<Loader>();
            let handle = loader.load_from_data(layout, (), &res.fetch());

            // TODO: This processing should be completely avoided. For that, we need
            // a better way to define computed assets.
            <Processor<EncodingLayout>>::new().run_now(&res);

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
