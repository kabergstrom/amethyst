use atelier_loader::{self, AssetTypeId, AssetUuid, Loader as AtelierLoader};
use crate::{Asset, AssetStorage, storage::{Processed, Handle}};
use amethyst_core::specs::Resources;
use serde_dyn::TypeUuid;
use serde::de::Deserialize;
use bincode;
use std::{
    error::Error,
    collections::HashMap,
    sync::Arc,
};

pub trait Loader: Send + Sync {
    type LoadOp;
    fn add_asset_ref(&mut self, id: AssetUuid) -> Self::LoadOp;
    fn get_asset_load(&self, id: &AssetUuid) -> Option<Self::LoadOp>;
    fn decrease_asset_ref(&mut self, id: AssetUuid);
    fn get_asset<T: Asset + TypeUuid>(&self, load: &Self::LoadOp) -> Option<Handle<T>>;
    fn init_world(&mut self, resources: &mut Resources);
    fn process(&mut self, resources: &Resources) -> Result<(), Box<dyn Error>>;
}

pub type DefaultLoader = LoaderWithStorage<atelier_loader::rpc_loader::RpcLoader<Arc<u32>>>;
#[derive(Default, Debug)]
pub struct LoaderWithStorage<T: AtelierLoader<HandleType = Arc<u32>> + Send + Sync> {
    loader: T,
    storage_map: AssetStorageMap,
}

impl<T: AtelierLoader<HandleType = Arc<u32>> + Send + Sync> Loader for LoaderWithStorage<T> {
    type LoadOp = <T as AtelierLoader>::LoadOp;
    fn add_asset_ref(&mut self, id: AssetUuid) -> Self::LoadOp {
        self.loader.add_asset_ref(id)
    }
    fn get_asset_load(&self, id: &AssetUuid) -> Option<Self::LoadOp> {
        self.loader.get_asset_load(id)
    }
    fn decrease_asset_ref(&mut self, id: AssetUuid) {
        self.loader.decrease_asset_ref(id)
    }
    fn get_asset<A: Asset + TypeUuid>(&self, load: &Self::LoadOp) -> Option<Handle<A>> {
        let uuid = A::UUID;
        let maybe_asset = self.loader.get_asset(load);
        if let Some((ref asset_type, ref asset_handle)) = maybe_asset {
            if asset_type != &uuid.to_le_bytes() {
                log::warn!("tried to fetch asset handle for type {} with uuid {:?} but type mismatched, expected uuid {:?}", A::name(), A::UUID, u128::from_le_bytes(*asset_type));
                return None
            } else {
                return maybe_asset.map(|(_, handle)| Handle::from_arc(handle))
            }
        }
        None
    }
    fn init_world(&mut self, resources: &mut Resources) {
        for (_, storage) in self.storage_map.storages.iter() {
            (storage.create_storage)(resources);
        }
    }
    fn process(&mut self, resources: &Resources) -> Result<(), Box<dyn Error>> {
        let storages = WorldStorages::new(resources, &self.storage_map);
        self.loader.process(&storages)
    }
}

pub trait AssetTypeStorage {
    fn allocate(&self) -> Arc<u32>;
    fn update_asset(&self, handle: &Arc<u32>, data: &dyn AsRef<[u8]>) -> Result<(), Box<dyn Error>>;
    fn is_loaded(&self, handle: &Arc<u32>) -> bool;
    fn free(&self, handle: Arc<u32>);
}
impl<A: Asset> AssetTypeStorage for AssetStorage<A> 
where for<'a> A::Data: Deserialize<'a> + TypeUuid {
    fn allocate(&self) -> Arc<u32> {
        self.allocate().id
    }
    fn update_asset(&self, handle: &Arc<u32>, data: &dyn AsRef<[u8]>) -> Result<(), Box<dyn Error>> {
        let asset = bincode::deserialize::<A::Data>(data.as_ref())?;
        self.processed.push(Processed::Asset {
            data: Ok(asset),
            handle: Handle::from_arc(Arc::clone(handle)),
            name: A::name().to_string(),
            tracker: None,
        });
        Ok(())
    }
    fn is_loaded(&self, handle: &Arc<u32>) -> bool {
        self.is_loaded(**handle)
    }
    fn free(&self, _handle: Arc<u32>) {
        // handle gets dropped since it's moved into this function
    }

}

#[derive(Debug)]
struct AssetStorageMap {
    pub storages: HashMap<AssetTypeId, AssetType>,
}

impl AssetStorageMap {
    pub fn new() -> AssetStorageMap {
        let mut asset_types = HashMap::new();
        for t in crate::inventory::iter::<AssetType> {
            asset_types.insert(t.data_uuid, t.clone());
        }
        AssetStorageMap {
            storages: asset_types,
        }
    }
}
impl Default for AssetStorageMap {
    fn default() -> Self {
        AssetStorageMap::new()
    }
}

struct WorldStorages<'a> {
    storage_map: &'a AssetStorageMap,
    res: &'a Resources,
}

impl<'a> WorldStorages<'a> {
    fn new(res: &'a Resources, storage_map: &'a AssetStorageMap) -> WorldStorages<'a> {
        WorldStorages {
            storage_map,
            res,
        }
    }
}

impl<'a> atelier_loader::AssetStorage for WorldStorages<'a> {
    type HandleType = Arc<u32>;
    fn allocate(&self, asset_type: &AssetTypeId, id: &AssetUuid) -> Self::HandleType {
        let mut handle = None;
        (self.storage_map.storages.get(asset_type).expect("could not find asset type").with_storage)(self.res, &mut |storage: &dyn AssetTypeStorage| {
            handle = Some(storage.allocate());
        });
        handle.unwrap()
    }
    fn update_asset(&self, asset_type: &AssetTypeId, handle: &Self::HandleType, data: &dyn AsRef<[u8]>) -> Result<(), Box<dyn Error>> {
        let mut result = None;
        (self.storage_map.storages.get(asset_type).expect("could not find asset type").with_storage)(self.res, &mut |storage: &dyn AssetTypeStorage| {
            result = Some(storage.update_asset(handle, data));
        });
        result.unwrap()
    }
    fn is_loaded(&self, asset_type: &AssetTypeId, handle: &Self::HandleType) -> bool {
        let mut loaded = None;
        (self.storage_map.storages.get(asset_type).expect("could not find asset type").with_storage)(self.res, &mut |storage: &dyn AssetTypeStorage| {
            loaded = Some(storage.is_loaded(handle))
        });
        loaded.unwrap()
    }
    fn free(&self, asset_type: &AssetTypeId, handle: Self::HandleType) {
        use std::cell::RefCell; // can't move into closure, so we work around it with a RefCell + Option
        let moved_handle = RefCell::new(Some(handle));
        (self.storage_map.storages.get(asset_type).expect("could not find asset type").with_storage)(self.res, &mut |storage: &dyn AssetTypeStorage| {
            storage.free(moved_handle.replace(None).unwrap())
        });
    }
}
#[derive(Clone)]
pub struct AssetType {
    pub data_uuid: AssetTypeId,
    pub asset_uuid: AssetTypeId,
    pub create_storage: fn(&mut Resources),
    pub with_storage: fn(&Resources, &mut dyn FnMut(&dyn AssetTypeStorage)),
}
impl std::fmt::Debug for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AssetType {{ data_uuid: {:?}, asset_uud: {:?} }}", self.data_uuid, self.asset_uuid)
    }
}
crate::inventory::collect!(AssetType);

pub fn create_asset_type<A: Asset + TypeUuid>() -> AssetType 
where for<'a> A::Data: Deserialize<'a> + TypeUuid {
    AssetType {
        data_uuid: A::Data::UUID.to_le_bytes(),
        asset_uuid: A::UUID.to_le_bytes(),
        create_storage: |res| {
            if res.try_fetch::<AssetStorage::<A>>().is_none() {
                res.insert(AssetStorage::<A>::default())
            }
        },
        with_storage: |res, func| func(&*res.fetch::<AssetStorage<A>>()),
    }
}


#[macro_export]
macro_rules! asset_type {
    ($($intermediate:ty => $asset:ty),*,) => {
        $crate::asset_type! {
            $(
                $intermediate => $asset
            ),*
        }
    };
    ($($intermediate:ty => $asset:ty),*) => {
        //mod asset_type_mod {
            use $crate::inventory;
            use $crate::create_asset_type;
            use super::*;
            $(
                inventory::submit! {
                    create_asset_type::<$asset>()
                }
            )*
        //}
    }
}