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
        prelude::{Component, System, VecStorage, Write},
        storage::UnprotectedStorage,
    },
};

use crate::{
    asset::{Asset, FormatValue},
    error::{Error, ErrorKind, Result, ResultExt},
    progress::Tracker,
};

/// An `Allocator`, holding a counter for producing unique IDs.
#[derive(Debug, Default)]
pub struct Allocator {
    store_count: AtomicUsize,
}

impl Allocator {
    /// Produces a new id.
    pub fn next_id(&self) -> usize {
        self.store_count.fetch_add(1, Ordering::Relaxed)
    }
}

/// An asset storage, storing the actual assets and allocating
/// handles to them.
pub struct AssetStorage<A: Asset> {
    assets: VecStorage<A>,
    bitset: BitSet,
    handles: Vec<Handle<A>>,
    handle_alloc: Allocator,
    pub(crate) processed: Arc<SegQueue<Processed<A>>>,
    unused_handles: SegQueue<Handle<A>>,
    requeue: Mutex<Vec<Processed<A>>>,
    to_drop: SegQueue<A>,
}

/// Returned by processor systems, describes the loading state of the asset.
pub enum ProcessingState<A>
where
    A: Asset,
{
    /// Asset is not fully loaded yet, need to wait longer
    Loading(A::Data),
    /// Asset have finished loading, can now be inserted into storage and tracker notified
    Loaded(A),
}

impl<A: Asset> AssetStorage<A> {
    /// Creates a new asset storage.
    pub fn new() -> Self {
        Default::default()
    }

    /// Allocate a new handle.
    pub(crate) fn allocate(&self) -> Handle<A> {
        self.unused_handles
            .try_pop()
            .unwrap_or_else(|| self.allocate_new())
    }

    fn allocate_new(&self) -> Handle<A> {
        let id = self.handle_alloc.next_id() as u32;
        Handle {
            id: Arc::new(id),
            marker: PhantomData,
        }
    }

    /// When cloning an asset handle, you'll get another handle,
    /// but pointing to the same asset. If you instead want to
    /// indeed create a new asset, you can use this method.
    /// Note however, that it needs a mutable borrow of `self`,
    /// so it can't be used in parallel.
    pub fn clone_asset(&mut self, handle: &Handle<A>) -> Option<Handle<A>>
    where
        A: Clone,
    {
        if let Some(asset) = self.get(handle).map(A::clone) {
            let h = self.allocate();

            let id = h.id();
            self.bitset.add(id);
            self.handles.push(h.clone());

            unsafe {
                self.assets.insert(id, asset);
            }

            Some(h)
        } else {
            None
        }
    }

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
    pub fn process<F>(
        &mut self,
        f: F,
    ) where
        F: FnMut(A::Data) -> Result<ProcessingState<A>>,
    {
        self.process_custom_drop(f, |_| {});
    }

    /// Process finished asset data and maintain the storage.
    /// This calls the `drop_fn` closure for assets that were removed from the storage.
    pub fn process_custom_drop<F, D>(
        &mut self,
        mut f: F,
        mut drop_fn: D,
    ) where
        D: FnMut(A),
        F: FnMut(A::Data) -> Result<ProcessingState<A>>,
    {
        {
            let requeue = self
                .requeue
                .get_mut()
                .expect("The mutex of `requeue` in `AssetStorage` was poisoned");
            while let Some(processed) = self.processed.try_pop() {
                let assets = &mut self.assets;
                let bitset = &mut self.bitset;
                let handles = &mut self.handles;

                let f = &mut f;
                match processed {
                    Processed::Asset {
                        data,
                        handle,
                        name,
                        tracker,
                    } => {
                        let asset = match data
                            .and_then(|d| f(d))
                            .chain_err(|| ErrorKind::Asset(name.clone()))
                        {
                            Ok(ProcessingState::Loaded(x)) => {
                                debug!(
                                        "{:?}: Asset {:?} (handle id: {:?}) has been loaded successfully",
                                        A::name(),
                                        name,
                                        handle,
                                    );
                                // Add a warning if a handle is unique (i.e. asset does not
                                // need to be loaded as it is not used by anything)
                                // https://github.com/amethyst/amethyst/issues/628
                                if handle.is_unique() {
                                    warn!(
                                        "Loading unnecessary asset. Handle {} is unique ",
                                        handle.id()
                                    );
                                    if let Some(tracker) = tracker {
                                        tracker.fail(
                                            handle.id(),
                                            A::name(),
                                            name,
                                            Error::from_kind(ErrorKind::UnusedHandle),
                                        );
                                    }
                                } else if let Some(tracker) = tracker {
                                    tracker.success();
                                }

                                x
                            }
                            Ok(ProcessingState::Loading(x)) => {
                                debug!(
                                        "{:?}: Asset {:?} (handle id: {:?}) is not complete, readding to queue",
                                        A::name(),
                                        name,
                                        handle,
                                    );
                                requeue.push(Processed::Asset {
                                    data: Ok(x),
                                    handle,
                                    name,
                                    tracker,
                                });
                                continue;
                            }
                            Err(e) => {
                                error!(
                                    "{:?}: Asset {:?} (handle id: {:?}) could not be loaded: {}",
                                    A::name(),
                                    name,
                                    handle,
                                    e,
                                );
                                if let Some(tracker) = tracker {
                                    tracker.fail(handle.id(), A::name(), name, e);
                                }

                                continue;
                            }
                        };
                        let id = handle.id();
                        if !bitset.add(id) {
                            // data doesn't exist for the handle, add it
                            handles.push(handle.clone());
                        } else {
                            unsafe {
                                self.to_drop.push(assets.remove(id));
                            }
                        }
                        unsafe {
                            assets.insert(id, asset);
                        }
                    }
                };
            }

            for p in requeue.drain(..) {
                self.processed.push(p);
            }
        }

        let mut count = 0;
        let mut skip = 0;
        while let Some(i) = self.handles.iter().skip(skip).position(Handle::is_unique) {
            count += 1;
            // Re-normalize index
            let i = skip + i;
            skip = i;
            let handle = self.handles.swap_remove(i);
            let id = handle.id();
            unsafe {
                self.to_drop.push(self.assets.remove(id));
            }
            self.bitset.remove(id);

            // Can't reuse old handle here, because otherwise weak handles would still be valid.
            // TODO: maybe just store u32?
            self.unused_handles.push(Handle {
                id: Arc::new(id),
                marker: PhantomData,
            });
        }
        while let Some(asset) = self.to_drop.try_pop() {
            drop_fn(asset);
        }
        if count != 0 {
            debug!("{:?}: Freed {} handle ids", A::name(), count,);
        }
    }
}

impl<A: Asset> Default for AssetStorage<A> {
    fn default() -> Self {
        AssetStorage {
            assets: Default::default(),
            bitset: Default::default(),
            handles: Default::default(),
            handle_alloc: Default::default(),
            processed: Arc::new(SegQueue::new()),
            unused_handles: SegQueue::new(),
            requeue: Mutex::new(Vec::default()),
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

/// A default implementation for an asset processing system
/// which converts data to assets and maintains the asset storage
/// for `A`.
///
/// This system can only be used if the asset data implements
/// `Into<Result<A, BoxedErr>>`.
pub struct Processor<A> {
    marker: PhantomData<A>,
}

impl<A> Processor<A> {
    /// Creates a new asset processor for
    /// assets of type `A`.
    pub fn new() -> Self {
        Processor {
            marker: PhantomData,
        }
    }
}

impl<'a, A> System<'a> for Processor<A>
where
    A: Asset,
    A::Data: Into<Result<ProcessingState<A>>>,
{
    type SystemData = Write<'a, AssetStorage<A>>;

    fn run(&mut self, mut storage: Self::SystemData) {
        storage.process(
            Into::into,
        );
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

pub(crate) enum Processed<A: Asset> {
    Asset {
        data: Result<A::Data>,
        handle: Handle<A>,
        name: String,
        tracker: Option<Box<dyn Tracker>>,
    },
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
