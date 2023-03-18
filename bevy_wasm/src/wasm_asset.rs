//! Implements loader for a custom asset type.

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, TypeUuid)]
#[uuid = "4e2a45df-246a-4ab8-91ac-c24218d6a79d"]
pub struct WasmAsset {
    pub bytes: Vec<u8>,
}

#[derive(Default)]
pub struct WasmAssetLoader;

impl AssetLoader for WasmAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
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
