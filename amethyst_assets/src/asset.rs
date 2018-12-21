use std::{ io::Read, sync::Arc, path::PathBuf };
use downcast::Any; 
use erased_serde::{Serialize};
use serde_dyn::{TypeUuid};
use ::uuid;

use amethyst_core::specs::storage::UnprotectedStorage;

use crate::{ErrorKind, Handle, Reload, Result, ResultExt, SingleFile, Source};


/// One of the three core traits of this crate.
///
/// You want to implement this for every type of asset like
///
/// * `Mesh`
/// * `Texture`
/// * `Terrain`
///
/// and so on. Now, an asset may be available in different formats.
/// That's why we have the `Data` associated type here. You can specify
/// an intermediate format here, like the vertex data for a mesh or the samples
/// for audio data.
///
/// This data is then generated by the `Format` trait.
pub trait Asset: Send + Sync + 'static {
    /// An identifier for this asset used for debugging.
    fn name() -> &'static str;

    /// The `Data` type the asset can be created from.
    type Data: Send + Sync + 'static;

    /// The ECS storage type to be used. You'll want to use `VecStorage` in most cases.
    type HandleStorage: UnprotectedStorage<Handle<Self>> + Send + Sync;
}

/// A format, providing a conversion from bytes to asset data, which is then
/// in turn accepted by `Asset::from_data`. Examples for formats are
/// `Png`, `Obj` and `Wave`.
pub trait Importer: Send + 'static {
    /// Version of the importer serialization format.
    fn version_static() -> u32 where Self: Sized;
    /// Version of the importer serialization format.
    fn version(&self) -> u32;
    /// Options specific to the format, which are passed to `import`.
    /// E.g. for textures this would be stuff like mipmap levels and
    /// sampler info.
    type Options: Send + 'static;

    /// State is specific to the format, which are passed to `import`.
    /// This is maintained by the asset pipeline to enable Importers to
    /// store state between calls to import().
    /// This is primarily used to store generated AssetIDs with mappings to
    /// format-internal identifiers and ensure IDs are stable between imports.
    type State: Serialize + Send + 'static;

    /// Reads the given bytes and produces asset data.
    fn import(
        &self,
        source: &mut dyn Read,
        options: Self::Options,
        state: &mut Self::State,
    ) -> Result<ImporterValue>;
}

/// AssetID is used to reference an asset
#[derive(Clone, Serialize, Deserialize, Hash)]
pub enum AssetID {
    /// Globally unique identifier for an asset. 
    /// Primary way to reference assets for tools and formats that are not edited by humans.
    /// Generated by the Asset Pipeline when the asset is imported.
    UUID(AssetUUID),
    /// A reference to a file on the local filesystem
    FilePath(PathBuf),
}
/// 16 byte v4 UUID for uniquely identifying imported assets
pub type AssetUUID = [u8; 16];
/// A trait for serializing any struct with a TypeUuid
pub trait SerdeObj: Any + Serialize + TypeUuid + Send {}
serialize_trait_object!(SerdeObj);
downcast!(dyn SerdeObj);
impl<T: Serialize + TypeUuid + Send + 'static> SerdeObj for T {}

/// Contains metadata and asset data for an imported asset
pub struct ImportedAsset {
    /// UUID for the asset to uniquely identify it
    pub id: AssetUUID,
    /// Search tags that are used by asset tooling to search for the imported asset
    pub search_tags: Vec<(String, Option<String>)>,
    /// Build dependencies will be included in the Builder arguments when building the asset
    pub build_deps: Vec<AssetID>,
    /// Load dependencies will be loaded before this asset in the Loader
    pub load_deps: Vec<AssetID>,
    /// Instantiate dependencies will be instantiated along with this asset when 
    /// the asset is instantiated into a world
    pub instantiate_deps: Vec<AssetID>,
    /// The actual asset data used by tools and Builder 
    pub asset_data: Box<dyn SerdeObj>,
}

/// Return value for Importers containing all imported assets
pub struct ImporterValue {
    /// All imported assets
    pub assets: Vec<ImportedAsset>,
}

/// A simple state for Importer to retain the same UUID between imports
/// for all single-asset source files
#[derive(Default, Serialize, Deserialize)]
pub struct SimpleImporterState {
    id: Option<AssetUUID>,
}

/// Wrapper struct to be able to impl Importer for any SimpleFormat
pub struct SimpleImporter<A: Asset, T: SimpleFormat<A>>(pub T, ::std::marker::PhantomData<A>);

impl<A: Asset, T: SimpleFormat<A> + 'static> From<T> for SimpleImporter<A, T> {
    fn from(fmt: T) -> SimpleImporter<A, T>{
        SimpleImporter(fmt, ::std::marker::PhantomData)
    }
}

impl<A: Asset, T:SimpleFormat<A> + Send + 'static> Importer for SimpleImporter<A, T>
where <A as Asset>::Data : SerdeObj
{
    type State = SimpleImporterState;
    type Options = T::Options;

    fn version_static() -> u32 where Self: Sized { 1 }
    fn version(&self) -> u32 { Self::version_static() }

    fn import(
        &self,
        source: &mut dyn Read,
        options: Self::Options,
        state: &mut Self::State,
    ) -> Result<ImporterValue> {
        if state.id.is_none() {
            state.id = Some(*uuid::Uuid::new_v4().as_bytes());
        }
        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes)
            .chain_err(|| format!("Failed to read bytes from source"))?;
        let import_result = self.0.import(bytes, options)?;
        Ok(ImporterValue {
            assets: vec![
                ImportedAsset {
                    id: state.id.expect("AssetID not generated"),
                    search_tags: Vec::new(),
                    build_deps: Vec::new(),
                    load_deps: Vec::new(),
                    instantiate_deps: Vec::new(),
                    asset_data: Box::new(import_result),
                }   
            ]
        })
    }
}

/// A format, providing a conversion from bytes to asset data, which is then
/// in turn accepted by `Asset::from_data`. Examples for formats are
/// `Png`, `Obj` and `Wave`.
pub trait Format<A: Asset>: Send + 'static {
    /// A unique identifier for this format.
    fn name() -> &'static str where Self: Sized;
    
    /// Options specific to the format, which are passed to `import`.
    /// E.g. for textures this would be stuff like mipmap levels and
    /// sampler info.
    type Options: Send + 'static;

    /// Reads the given bytes and produces asset data.
    ///
    /// ## Reload
    ///
    /// The reload structure has metadata which allows the asset management
    /// to reload assets if necessary (for hot reloading).
    /// You should only create this if `create_reload` is `true`.
    /// Also, the parameter is just a request, which means you can also return `None`.
    fn import(
        &self,
        name: String,
        source: Arc<dyn Source>,
        options: Self::Options,
        create_reload: bool,
    ) -> Result<FormatValue<A>>;
}

/// The `Ok` return value of `Format::import` for a given asset type `A`.
pub struct FormatValue<A: Asset> {
    /// The format data.
    pub data: A::Data,
    /// An optional reload structure
    pub reload: Option<Box<dyn Reload<A>>>,
}

impl<A: Asset> FormatValue<A> {
    /// Creates a `FormatValue` from only the data (setting `reload` to `None`).
    pub fn data(data: A::Data) -> Self {
        FormatValue { data, reload: None }
    }
}

/// This is a simplified version of `Format`, which doesn't give you as much freedom,
/// but in return is simpler to implement.
/// All `SimpleFormat` types automatically implement `Format`.
/// This format assumes that the asset name is the full path and the asset is only
/// contained in one file.
pub trait SimpleFormat<A: Asset> {
    /// A unique identifier for this format.
    fn name() -> &'static str where Self:Sized { "NONE" } 
    /// Options specific to the format, which are passed to `import`.
    /// E.g. for textures this would be stuff like mipmap levels and
    /// sampler info.
    type Options: Clone + Send + Sync + 'static;

    /// Produces asset data from given bytes.
    fn import(&self, bytes: Vec<u8>, options: Self::Options) -> Result<A::Data>;
}

impl<A, T> Format<A> for T
where
    A: Asset,
    T: SimpleFormat<A> + Clone + Send + Sync + 'static,
{
    fn name() -> &'static str { T::name() }
    type Options = T::Options;

    fn import(
        &self,
        name: String,
        source: Arc<dyn Source>,
        options: Self::Options,
        create_reload: bool,
    ) -> Result<FormatValue<A>> {
        #[cfg(feature = "profiler")]
        profile_scope!("import_asset");
        if create_reload {
            let (b, m) = source
                .load_with_metadata(&name)
                .chain_err(|| ErrorKind::Source)?;
            let data = T::import(&self, b, options.clone())?;
            let reload = SingleFile::new(self.clone(), m, options, name, source);
            let reload = Some(Box::new(reload) as Box<dyn Reload<A>>);
            Ok(FormatValue { data, reload })
        } else {
            let b = source.load(&name).chain_err(|| ErrorKind::Source)?;
            let data = T::import(&self, b, options)?;

            Ok(FormatValue::data(data))
        }
    }
}

uuid!{
    SimpleImporterState => 276663539928909366810068622540168088635
}