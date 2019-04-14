use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, Weak,
    },
};

use crossbeam::queue::SegQueue;
use derivative::Derivative;
use hibitset::BitSet;
use log::{debug, error, trace, warn};
use atelier_loader::{LoadHandle, slotmap};

use amethyst_core::{
    specs::{
        prelude::{Component, System, DenseVecStorage, Write},
        storage::UnprotectedStorage,
    },
};

use crate::{
    asset::{Asset, FormatValue},
    error::{Error, ErrorKind, Result, ResultExt},
    new_loader::{AssetHandle},
    progress::Tracker,
};

struct AssetState<A> {
    version: u32,
    asset: A,
}
/// An asset storage, storing the actual assets and allocating
/// handles to them.
pub struct AssetStorage<A: Asset> {
    assets: slotmap::SecondaryMap<LoadHandle, AssetState<A>>,
    to_drop: SegQueue<A>,
}

impl<A: Asset> AssetStorage<A> {
    /// Creates a new asset storage.
    pub fn new() -> Self {
        Default::default()
    }


    pub(crate) fn update_asset(&mut self, handle: &LoadHandle, asset: A) {
        let mut version = 0;
        if let Some(data) = self.assets.remove(*handle) {
            // data already exists for the handle, drop it
            self.to_drop.push(data.asset);
            version = data.version;
        }
        self.assets.insert(*handle, AssetState {
            version: version + 1,
            asset,
        });
    }

    // TODO implement this as a pub(crate) function for usage by Loader and let Loader manage handle allocation

    // /// When cloning an asset handle, you'll get another handle,
    // /// but pointing to the same asset. If you instead want to
    // /// indeed create a new asset, you can use this method.
    // /// Note however, that it needs a mutable borrow of `self`,
    // /// so it can't be used in parallel.
    // pub fn clone_asset(&mut self, handle: &Handle<A>) -> Option<Handle<A>>
    // where
    //     A: Clone,
    // {
    //     if let Some(asset) = self.get(handle).map(A::clone) {
    //         let h = self.allocate();

    //         let id = h.id();
    //         self.bitset.add(id);
    //         self.handles.push(h.clone());

    //         unsafe {
    //             self.assets.insert(id, asset);
    //         }

    //         Some(h)
    //     } else {
    //         None
    //     }
    // }

    pub(crate) fn is_loaded<T: AssetHandle>(&self, handle: &T) -> bool {
        self.assets.contains_key(*handle.get_load_handle())
    }

    /// Get an asset from a given asset handle.
    pub fn get<T: AssetHandle>(&self, handle: &T) -> Option<&A> {
        self.assets.get(*handle.get_load_handle()).map(|a| &a.asset)
    }

    /// Get an asset mutably from a given asset handle.
    pub fn get_mut<T: AssetHandle>(&mut self, handle: &T) -> Option<&mut A> {
        self.assets.get_mut(*handle.get_load_handle()).map(|a| &mut a.asset)
    }

    pub fn get_version<T: AssetHandle>(&self, handle: &T) -> Option<u32> {
        self.assets.get(*handle.get_load_handle()).map(|a| a.version)
    }

    pub fn get_asset_with_version<T: AssetHandle>(&self, handle: &T) -> Option<(&A, u32)> {
        self.assets.get(*handle.get_load_handle()).map(|a| (&a.asset, a.version))
    }

    /// Process finished asset data and maintain the storage.
    /// This calls the `drop_fn` closure for assets that were removed from the storage.
    pub fn process_custom_drop<D>(
        &mut self,
        mut drop_fn: D,
    ) where
        D: FnMut(A),
    {
        while let Some(asset) = self.to_drop.try_pop() {
            drop_fn(asset);
        }
    }
}

impl<A: Asset> Default for AssetStorage<A> {
    fn default() -> Self {
        AssetStorage {
            assets: Default::default(),
            to_drop: SegQueue::new(),
        }
    }
}

