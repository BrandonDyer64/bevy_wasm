//! Implements loader for a custom asset type.
use bevy::{
    asset::{io::Reader, AssetLoader, LoadedAsset, LoadContext},
    prelude::*,
    reflect::TypePath,
    tasks::BoxedFuture,
};
use thiserror::Error;

use tracing::{error, info, warn};
use serde::Deserialize;
use bevy_wasm_shared::resource_id::{ResourceId, resource_id};
// #[uuid = "4e2a45df-246a-4ab8-91ac-c24218d6a79d"]
#[derive(Asset, Debug, Deserialize, TypePath)]
pub struct WasmAsset {
    pub bytes: Vec<u8>,
}
impl WasmAsset {
    /// Deterministic resource ID for WASM asset system
    pub fn get_resource_id() -> ResourceId {
        resource_id::<WasmAsset>()
    }
}


#[derive(Default, TypePath)]
pub struct WasmAssetLoader;
/*
impl AssetLoader for WasmAssetLoader {

    type Asset = WasmAsset;
    type Settings = ();
    type Error = CustomAssetLoaderError;

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
       // load_context: &'a mut LoadContext,
        load_context: &mut LoadContext<'_>,
        //  ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
    ) -> BoxedFuture<'a, Result<(), Self::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(WasmAsset {
                bytes: bytes.into(),
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["wasm"]
    }
}
*/
/*
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CustomAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
}
*/
/*
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CustomAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
}
*/
impl AssetLoader for WasmAssetLoader {
    type Asset = WasmAsset;
    type Settings = ();
    type Error = CustomAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(WasmAsset { bytes })
    }

    fn extensions(&self) -> &[&str] {
        &["wasm"]
    }
}
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CustomAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
}
