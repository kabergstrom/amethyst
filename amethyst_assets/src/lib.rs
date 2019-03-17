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
    asset::{Asset, Format, FormatValue, SimpleFormat, AssetUUID, importer, SimpleImporter},
    cache::Cache,
    error::{Error, ErrorKind, Result, ResultExt},
    formats::RonFormat,
    helper::AssetLoaderSystemData,
    loader::Loader,
    prefab::{AssetPrefab, Prefab, PrefabData, PrefabError, PrefabLoader, PrefabLoaderSystem},
    progress::{Completion, Progress, ProgressCounter, Tracker},
    reload::{HotReloadBundle, HotReloadStrategy, HotReloadSystem, Reload, SingleFile},
    source::{Directory, Source},
    storage::{AssetStorage, Handle, WeakHandle},
    processor::{ProcessingState, Processor},
    new_loader::{create_asset_type, DefaultLoader, Loader as NewLoader},
};
pub use atelier_importer::inventory;


mod asset;
mod cache;
mod error;
mod formats;
mod helper;
mod loader;
mod prefab;
mod progress;
mod reload;
mod source;
mod storage;
mod processor;
#[macro_use]
mod new_loader;