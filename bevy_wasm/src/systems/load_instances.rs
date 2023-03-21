use bevy::prelude::*;

use crate::{
    components::WasmMod,
    runtime::{WasmInstance, WasmRuntime},
    wasm_asset::WasmAsset,
};

pub fn load_instances(
    mut commands: Commands,
    wasm_assets: Res<Assets<WasmAsset>>,
    mods_to_load: Query<(Entity, &WasmMod), Without<WasmInstance>>,
    wasm_runtime: Res<WasmRuntime>,
) {
    for (entity, mod_to_load) in mods_to_load.iter() {
        if let Some(wasm_asset) = wasm_assets.get(&mod_to_load.wasm) {
            let instance = wasm_runtime.create_instance(&wasm_asset.bytes);
            match instance {
                Ok(instance) => {
                    commands.entity(entity).insert(instance);
                }
                Err(e) => {
                    error!("Could not initialize WASM instance: {}", e);
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
