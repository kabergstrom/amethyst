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
pub type DefaultLoader = atelier_loader::rpc_loader::RpcLoader<WorldStorages>;

pub struct AssetStorageMap {
    storages: HashMap<AssetTypeId, AssetType>,
}

impl AssetStorageMap {
    pub fn new() -> AssetStorageMap {
        AssetStorageMap {
            storages: HashMap::new(),
        }
    }
}

struct WorldStorages {
    storage_map: AssetStorageMap,
    world: &World,
}

impl atelier_loader::AssetStorage for WorldStorages {
    type HandleType = u32;
    fn allocate(&self, asset_type: &AssetTypeId, id: &AssetUuid) -> Self::HandleType {
        0
    }
    fn update_asset(&self, asset_type: &AssetTypeId, handle: &Self::HandleType, data: &dyn AsRef<[u8]>) -> Result<(), Box<dyn Error>> {
        // let asset = bincode::deserialize::<A::Data>(data.as_ref())?;
        Ok(())
    }
    fn free(&self, asset_type: &AssetTypeId, handle: Self::HandleType) {

    }
}

pub struct AssetType {
    pub uuid: u128,
    // pub create_storage: fn() -> Box<dyn AssetStorage>,
}
inventory::collect!(AssetType);


pub fn create_asset_type<A: Asset>() -> AssetType 
where for<'a> A::Data: Deserialize<'a> + TypeUuid
{
    AssetType {
        uuid: A::Data::UUID,
        // create_storage: || Box::new(AssetStorage::<A>::default()),
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