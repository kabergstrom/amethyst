use atelier_loader::{self, AssetTypeId, AssetUuid};
use crate::{Asset, AssetStorage};
use amethyst_core::specs::World;
use serde_dyn::TypeUuid;
use serde::de::Deserialize;
use bincode;
use std::{
    error::Error,
    collections::HashMap,
};

pub use atelier_loader::Loader;
pub type DefaultLoader = atelier_loader::rpc_loader::RpcLoader<u32>;

pub trait AssetTypeStorage {
    fn allocate(&self) -> u32;
    fn update_asset(&self, handle: u32, data: &dyn AsRef<[u8]>) -> Result<(), Box<dyn Error>>;
    fn free(&self, handle: u32);
}
impl<A: Asset> AssetTypeStorage for AssetStorage<A> 
where for<'a> A::Data: Deserialize<'a> + TypeUuid {
    fn allocate(&self) -> u32 {
        0
    }
    fn update_asset(&self, handle: u32, data: &dyn AsRef<[u8]>) -> Result<(), Box<dyn Error>> {
        let asset = bincode::deserialize::<A::Data>(data.as_ref())?;
        println!("got asset");
        Ok(())
    }
    fn free(&self, handle: u32) {

    }

}

struct AssetStorageMap {
    pub storages: HashMap<AssetTypeId, AssetType>,
}

impl AssetStorageMap {
    pub fn new() -> AssetStorageMap {
        let mut asset_types = HashMap::new();
        for t in inventory::iter::<AssetType> {
            asset_types.insert(t.uuid, t.clone());
        }
        AssetStorageMap {
            storages: asset_types,
        }
    }
}

pub struct WorldStorages<'a> {
    storage_map: AssetStorageMap,
    world: &'a World,
}

impl<'a> WorldStorages<'a> {
    pub fn new(world: &'a World) -> WorldStorages {
        WorldStorages {
            storage_map: AssetStorageMap::new(),
            world,
        }
    }
}

impl<'a> atelier_loader::AssetStorage for WorldStorages<'a> {
    type HandleType = u32;
    fn allocate(&self, asset_type: &AssetTypeId, id: &AssetUuid) -> Self::HandleType {
        let mut handle = None;
        (self.storage_map.storages.get(asset_type).expect("could not find asset type").with_storage)(self.world, &mut |storage: &dyn AssetTypeStorage| {
            handle = Some(storage.allocate());
        });
        handle.unwrap()
    }
    fn update_asset(&self, asset_type: &AssetTypeId, handle: &Self::HandleType, data: &dyn AsRef<[u8]>) -> Result<(), Box<dyn Error>> {
        let mut result = None;
        (self.storage_map.storages.get(asset_type).expect("could not find asset type").with_storage)(self.world, &mut |storage: &dyn AssetTypeStorage| {
            result = Some(storage.update_asset(*handle, data));
        });
        result.unwrap()
    }
    fn free(&self, asset_type: &AssetTypeId, handle: Self::HandleType) {
        (self.storage_map.storages.get(asset_type).expect("could not find asset type").with_storage)(self.world, &mut |storage: &dyn AssetTypeStorage| {
            storage.free(handle)
        });
    }
}
#[derive(Clone)]
pub struct AssetType {
    pub uuid: AssetTypeId,
    pub with_storage: fn(&World, &mut dyn FnMut(&dyn AssetTypeStorage)),
}
inventory::collect!(AssetType);

pub fn create_asset_type<A: Asset>() -> AssetType 
where for<'a> A::Data: Deserialize<'a> + TypeUuid {
    AssetType {
        uuid: A::Data::UUID.to_le_bytes(),
        with_storage: |world, func| func(&*world.read_resource::<AssetStorage<A>>()),
    }
}


#[macro_export]
macro_rules! asset_type {
    ($($type:ty),*) => {
        $(
            use amethyst_assets::inventory;
            inventory::submit! {
                amethyst_assets::create_asset_type::<$type>()
            }
        )*
    }
}