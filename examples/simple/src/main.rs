use bevy::{log::LogPlugin, prelude::*};
use bevy_wasm::*;

pub static SCRIPTS_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/scripts.wasm"));

fn main() {
    App::new()
        .add_plugin(LogPlugin::default())
        .add_plugins(MinimalPlugins)
        .add_plugin(WasmPlugin(vec![SCRIPTS_WASM.into()]))
        .add_system(hello_world_system)
        .run();
}

fn hello_world_system() {
    info!("Hello, world from outside wasm!");
}
