//! # amethyst_assets
//!
//! Asset management crate.
//! Designed with the following goals in mind:
//!
//! * Extensibility
//! * Asynchronous & Parallel using Rayon
//! * Allow different sources

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

#[cfg(feature = "json")]
pub use crate::formats::JsonFormat;
pub use crate::{
    asset::{importer, Asset, AssetUUID, Format, FormatValue, SimpleFormat, SimpleImporter},
    cache::Cache,
    error::{Error, ErrorKind, Result, ResultExt},
    formats::RonFormat,
    helper::AssetLoaderSystemData,
    loader::Loader,
    new_loader::{create_asset_type, DefaultLoader, Loader as NewLoader, GenericHandle, AssetHandle},
    prefab::{AssetPrefab, Prefab, PrefabData, PrefabError, PrefabLoader, PrefabLoaderSystem},
    processor::{ProcessingState as NewProcessingState, Processor, ProcessingQueue},
    progress::{Completion, Progress, ProgressCounter, Tracker},
    reload::{HotReloadBundle, HotReloadStrategy, HotReloadSystem, Reload, SingleFile},
    source::{Directory, Source},
    storage::{AssetStorage, Handle, ProcessingState, WeakHandle},
    new_storage::{AssetStorage as NewAssetStorage},
};
pub use atelier_importer::inventory;

mod asset;
mod cache;
mod error;
mod formats;
mod helper;
mod loader;
mod prefab;
mod processor;
mod progress;
mod reload;
mod source;
mod storage;
#[macro_use]
mod new_loader;
mod new_storage;
