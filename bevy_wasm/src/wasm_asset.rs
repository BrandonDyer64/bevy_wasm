//! Implements loader for a custom asset type.

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::TypePath;
use serde::Deserialize;
use thiserror::Error;
use type_uuid::TypeUuid;

#[derive(Asset, Debug, Deserialize, TypeUuid, TypePath)]
#[uuid = "4e2a45df-246a-4ab8-91ac-c24218d6a79d"]
pub struct WasmAsset {
    pub bytes: Vec<u8>,
}

#[derive(Default)]
pub struct WasmAssetLoader;

#[derive(Error, Debug)]
pub enum WasmAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for WasmAssetLoader {
    type Asset = WasmAsset;
    type Settings = ();
    type Error = WasmAssetLoaderError;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset = WasmAsset { bytes };
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["wasm"]
    }
}
