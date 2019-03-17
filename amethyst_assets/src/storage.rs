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

use amethyst_core::{
    specs::{
        prelude::{Component, System, DenseVecStorage, Write},
        storage::UnprotectedStorage,
    },
};

use crate::{
    asset::{Asset, FormatValue},
    error::{Error, ErrorKind, Result, ResultExt},
    progress::Tracker,
};

/// An asset storage, storing the actual assets and allocating
/// handles to them.
pub struct AssetStorage<A: Asset> {
    assets: DenseVecStorage<A>,
    bitset: BitSet,
    to_drop: SegQueue<A>,
}

impl<A: Asset> AssetStorage<A> {
    /// Creates a new asset storage.
    pub fn new() -> Self {
        Default::default()
    }


    pub(crate) fn update_asset(&mut self, handle: u32, asset: A) {
        if self.bitset.add(handle) {
            // data already exists for the handle, drop it
            unsafe {
                self.to_drop.push(self.assets.remove(handle));
            }
        }
        unsafe {
            self.assets.insert(handle, asset);
        }
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

    pub(crate) fn is_loaded(&self, handle: u32) -> bool {
        self.bitset.contains(handle)
    }

    /// Get an asset from a given asset handle.
    pub fn get(&self, handle: &Handle<A>) -> Option<&A> {
        if self.bitset.contains(handle.id()) {
            Some(unsafe { self.assets.get(handle.id()) })
        } else {
            None
        }
    }

    /// Get an asset mutably from a given asset handle.
    pub fn get_mut(&mut self, handle: &Handle<A>) -> Option<&mut A> {
        if self.bitset.contains(handle.id()) {
            Some(unsafe { self.assets.get_mut(handle.id()) })
        } else {
            None
        }
    }

    /// Process finished asset data and maintain the storage.
    /// This calls the `drop_fn` closure for assets that were removed from the storage.
    pub fn process_custom_drop<F, D>(
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
            bitset: Default::default(),
            to_drop: SegQueue::new(),
        }
    }
}

impl<A: Asset> Drop for AssetStorage<A> {
    fn drop(&mut self) {
        let bitset = &self.bitset;
        unsafe { self.assets.clean(bitset) }
    }
}


/// A handle to an asset. This is usually what the
/// user deals with, the actual asset (`A`) is stored
/// in an `AssetStorage`.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Debug(bound = "")
)]
pub struct Handle<A: ?Sized> {
    pub(crate) id: Arc<u32>,
    #[derivative(Debug = "ignore")]
    marker: PhantomData<A>,
}

impl<A> Handle<A> {
    /// Return the 32 bit id of this handle.
    pub fn id(&self) -> u32 {
        *self.id.as_ref()
    }

    pub(crate) fn from_arc(id: Arc<u32>) -> Self {
        Self { 
            id,
            marker: PhantomData,
        }
    }

    /// Downgrades the handle and creates a `WeakHandle`.
    pub fn downgrade(&self) -> WeakHandle<A> {
        let id = Arc::downgrade(&self.id);

        WeakHandle {
            id,
            marker: PhantomData,
        }
    }

    /// Returns `true` if this is the only handle to the asset its pointing at.
    fn is_unique(&self) -> bool {
        Arc::strong_count(&self.id) == 1
    }
}

impl<A> Component for Handle<A>
where
    A: Asset,
{
    type Storage = A::HandleStorage;
}

/// A weak handle, which is useful if you don't directly need the asset
/// like in caches. This way, the asset can still get dropped (if you want that).
#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct WeakHandle<A> {
    id: Weak<u32>,
    marker: PhantomData<A>,
}

impl<A> WeakHandle<A> {
    /// Tries to upgrade to a `Handle`.
    #[inline]
    pub fn upgrade(&self) -> Option<Handle<A>> {
        self.id.upgrade().map(|id| Handle {
            id,
            marker: PhantomData,
        })
    }

    /// Returns `true` if the original handle is dead.
    #[inline]
    pub fn is_dead(&self) -> bool {
        self.upgrade().is_none()
    }
}
