use crate::{
    storage::{Handle},
    processor::{Processed, ProcessingQueue},
    Asset, AssetStorage,
};
use amethyst_core::specs::Resources;
use atelier_loader::{self, AssetTypeId, AssetUuid, Loader as AtelierLoader};
use bincode;
use crossbeam::channel::{unbounded, Receiver, Sender};
use serde::de::Deserialize;
use serde_dyn::TypeUuid;
use std::{collections::HashMap, error::Error, sync::Arc};

enum LoadStatus {
    NotRequested,
    Loading,
    Error(amethyst_error::Error),
    Loaded,
    DoesNotExist,
}

struct LoadHandle {
    chan: Arc<Sender<RefOp>>,
    id: u32,
}
impl AssetHandle for LoadHandle {
    fn get_id(&self) -> u32 {
        self.id
    }
}

struct WeakHandle {
    load_handle: u32,
}
impl AssetHandle for WeakHandle {  }

trait AssetHandle {
    fn get_load_status<T: Loader>(&self, loader: &T) -> LoadStatus {
        loader.get_load_status(self.get_id())
    }
    fn get_asset<'a, T: Asset + TypeUuid>(&self, storage: &'a AssetStorage<T>) -> Option<&'a T>;
    fn get_asset_mut<'a, T: Asset + TypeUuid>(&self, storage: &'a mut AssetStorage<T>) -> Option<&'a mut T>;
    fn get_version<'a, T: Asset + TypeUuid>(&self, storage: &'a AssetStorage<T>) -> u32;
    fn get_asset_with_version<'a, T: Asset + TypeUuid>(&self, storage: &'a AssetStorage<T>) -> Option<(&'a T, u32)>;
    fn get_id(&self) -> u32;
}

pub(crate) trait LoaderInternal {
    fn load_asset(&self, id: u32) -> LoadHandle;
    fn get_handle_load_status(&self, id: u32) -> LoadStatus;
    fn get_asset_handle<T: Asset + TypeUuid>(&self, id: u32) -> Option<Handle<T>>;
    fn get_asset<'a, T: Asset + TypeUuid>(&self, id: u32, storage: &'a AssetStorage<T>) -> Option<&'a T>;
    fn get_asset_mut<'a, T: Asset + TypeUuid>(&self, id: u32, storage: &'a mut AssetStorage<T>) -> Option<&'a mut T>;
    fn init_world(&mut self, resources: &mut Resources);
    fn process(&mut self, resources: &Resources) -> Result<(), Box<dyn Error>>;
}

pub trait Loader: Send + Sync + LoaderInternal {
    fn load_asset(&self, id: AssetUuid) -> LoadHandle;
    fn get_load_status(&self, id: AssetUuid) -> LoadStatus;
    fn get_asset_handle<T: Asset + TypeUuid>(&self, id: AssetUuid) -> Option<Handle<T>>;
    fn get_asset<'a, T: Asset + TypeUuid>(&self, id: AssetUuid, storage: &'a AssetStorage<T>) -> Option<&'a T>;
    fn get_asset_mut<'a, T: Asset + TypeUuid>(&self, id: AssetUuid, storage: &'a mut AssetStorage<T>) -> Option<&'a mut T>;
    fn init_world(&mut self, resources: &mut Resources);
    fn process(&mut self, resources: &Resources) -> Result<(), Box<dyn Error>>;
}

pub type DefaultLoader = LoaderWithStorage<atelier_loader::rpc_loader::RpcLoader<()>>;
enum RefOp {
    Increase(AssetUuid),
    Decrease(AssetUuid),
}
#[derive(Debug)]
pub struct LoaderWithStorage<T: AtelierLoader<HandleType = ()> + Send + Sync> {
    loader: T,
    storage_map: AssetStorageMap,
    ref_sender: Sender<RefOp>,
    ref_receiver: Receiver<RefOp>,
}
impl<T: AtelierLoader<HandleType = ()> + Send + Sync + Default> Default
    for LoaderWithStorage<T>
{
    fn default() -> Self {
        let (tx, rx) = unbounded();
        Self {
            loader: Default::default(),
            storage_map: Default::default(),
            ref_sender: tx,
            ref_receiver: rx,
        }
    }
}

impl<T: AtelierLoader<HandleType = ()> + Send + Sync> Loader for LoaderWithStorage<T> {
    fn add_asset_ref(&self, id: AssetUuid) {
        self.ref_sender.send(RefOp::Increase(id));
    }
    fn decrease_asset_ref(&self, id: AssetUuid) {
        self.ref_sender.send(RefOp::Decrease(id));
    }
    fn get_asset_handle<A: Asset + TypeUuid>(&self, id: AssetUuid) -> Option<Handle<A>> {
        let asset_type = A::UUID;
        let maybe_asset = self.loader.get_asset(id);
        if let Some((ref data_type, ref asset_handle)) = maybe_asset {
            if let Some(storage_type) = self.storage_map.storages_by_data_uuid.get(&data_type.to_le_bytes()) {
            if asset_type != &storage_type.asset_uuid {
                log::warn!("tried to fetch asset handle for type {} with uuid {:?} but type mismatched, expected uuid {:?}", A::name(), A::UUID, asset_type);
                return None;
            } else {
                return Some(Handle::from_arc(asset_handle.clone()));
            }

            } else {
                log::warn!("tried to fetch asset handle for type {} with uuid {:?} but the storage is not registered", A::name(), A::UUID);
            }

        }
        None
    }
    fn get_asset<'a, A: Asset + TypeUuid>(&self, id: AssetUuid, storage: &'a AssetStorage<A>) -> Option<&'a A> {
        if let Some(ref handle) = self.get_asset_handle(id) {
            storage.get(handle)
        } else {
            None
        }
    }
    fn get_asset_mut<'a, A: Asset + TypeUuid>(&self, id: AssetUuid, storage: &'a mut AssetStorage<A>) -> Option<&'a mut A> {
        if let Some(ref handle) = self.get_asset_handle(id) {
            storage.get_mut(handle)
        } else {
            None
        }
    }
    fn init_world(&mut self, resources: &mut Resources) {
        for (_, storage) in self.storage_map.storages_by_asset_uuid.iter() {
            (storage.create_storage)(resources);
        }
    }
    fn process(&mut self, resources: &Resources) -> Result<(), Box<dyn Error>> {
        loop {
            match self.ref_receiver.try_recv() {
                None => break,
                Some(RefOp::Increase(id)) => { self.loader.add_asset_ref(id);  },
                Some(RefOp::Decrease(id)) => self.loader.decrease_asset_ref(id),
            }
        }
        let storages = WorldStorages::new(resources, &self.storage_map);
        self.loader.process(&storages)
    }
}

pub trait AssetTypeStorage {
    fn allocate(&self) -> Arc<u32>;
    fn update_asset(&self, handle: &Arc<u32>, data: &dyn AsRef<[u8]>)
        -> Result<(), Box<dyn Error>>;
    fn is_loaded(&self, handle: &Arc<u32>) -> bool;
    fn free(&self, handle: Arc<u32>);
}
impl<A: Asset> AssetTypeStorage for AssetStorage<A>
where
    for<'a> A::Data: Deserialize<'a> + TypeUuid,
{
    fn allocate(&self) -> Arc<u32> {
        self.allocate().id
    }
    fn update_asset(
        &self,
        handle: &Arc<u32>,
        data: &dyn AsRef<[u8]>,
    ) -> Result<(), Box<dyn Error>> {
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
    pub storages_by_data_uuid: HashMap<AssetTypeId, AssetType>,
    pub storages_by_asset_uuid: HashMap<AssetTypeId, AssetType>,
}

impl AssetStorageMap {
    pub fn new() -> AssetStorageMap {
        let mut storages_by_asset_uuid = HashMap::new();
        let mut storages_by_data_uuid = HashMap::new();
        for t in crate::inventory::iter::<AssetType> {
            storages_by_data_uuid.insert(t.data_uuid, t.clone());
            storages_by_asset_uuid.insert(t.asset_uuid, t.clone());
        }
        AssetStorageMap {
            storages_by_asset_uuid,
            storages_by_data_uuid,
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
        WorldStorages { storage_map, res }
    }
}

impl<'a> atelier_loader::AssetStorage for WorldStorages<'a> {
    type HandleType = ();
    fn allocate(&self, asset_type: &AssetTypeId, id: &AssetUuid) -> Self::HandleType {
        let mut handle = None;
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.res, &mut |storage: &dyn AssetTypeStorage| {
            handle = Some(storage.allocate());
        });
        handle.unwrap()
    }
    fn update_asset(
        &self,
        asset_type: &AssetTypeId,
        handle: &Self::HandleType,
        data: &dyn AsRef<[u8]>,
    ) -> Result<(), Box<dyn Error>> {
        let mut result = None;
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.res, &mut |storage: &dyn AssetTypeStorage| {
            result = Some(storage.update_asset(handle, data));
        });
        result.unwrap()
    }
    fn is_loaded(&self, asset_type: &AssetTypeId, handle: &Self::HandleType) -> bool {
        let mut loaded = None;
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.res, &mut |storage: &dyn AssetTypeStorage| {
            loaded = Some(storage.is_loaded(handle))
        });
        loaded.unwrap()
    }
    fn free(&self, asset_type: &AssetTypeId, handle: Self::HandleType) {
        use std::cell::RefCell; // can't move into closure, so we work around it with a RefCell + Option
        let moved_handle = RefCell::new(Some(handle));
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.res, &mut |storage: &dyn AssetTypeStorage| {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AssetType {{ data_uuid: {:?}, asset_uud: {:?} }}",
            self.data_uuid, self.asset_uuid
        )
    }
}
crate::inventory::collect!(AssetType);

pub fn create_asset_type<A: Asset + TypeUuid>() -> AssetType
where
    for<'a> A::Data: Deserialize<'a> + TypeUuid,
{
    AssetType {
        data_uuid: A::Data::UUID.to_le_bytes(),
        asset_uuid: A::UUID.to_le_bytes(),
        create_storage: |res| {
            if res.try_fetch::<AssetStorage<A>>().is_none() {
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
