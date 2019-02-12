use atelier_loader;
use crate::{Asset, AssetStorage};
use serde_dyn::TypeUuid;
use serde::de::Deserialize;
use bincode;
use std::error::Error;
pub use atelier_loader::{inventory, Loader, DefaultLoader};

impl<A: Asset> atelier_loader::AssetStorage for AssetStorage<A> 
where for<'a> A::Data: Deserialize<'a> 
{
    fn allocate(&self) -> u32 {
        self.allocate().id()
    }
    fn update_asset(&self, handle: u32, data: &dyn AsRef<[u8]>) -> Result<(), Box<dyn Error>>
    {
        let asset = bincode::deserialize::<A::Data>(data.as_ref())?;
        Ok(())
    }
    fn free(&self, handle: u32) {

    }
}

pub fn create_asset_type<A: Asset>() -> atelier_loader::AssetType 
where for<'a> A::Data: Deserialize<'a> + TypeUuid
{
    atelier_loader::AssetType {
        uuid: A::Data::UUID,
        create_storage: || Box::new(AssetStorage::<A>::default()),
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