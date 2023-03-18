use bevy::prelude::*;

use crate::wasm_asset::WasmAsset;

/// The [`WasmMod`] component is used to spawn a new WebAssembly Mod into the world
///
/// # Example
///
/// ```
/// commands.spawn(WasmMod {
///     wasm: asset_server.load("my_mod.wasm"),
/// });
/// ```
#[derive(Component)]
pub struct WasmMod {
    /// Handle to the underlying WebAssembly binary
    pub wasm: Handle<WasmAsset>,
}
