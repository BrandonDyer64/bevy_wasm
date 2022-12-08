use bevy::{log::LogPlugin, prelude::*};
use bevy_wasm::*;
use serde::{Deserialize, Serialize};

pub static SCRIPTS_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/scripts.wasm"));

#[derive(Debug, Serialize, Deserialize)]
struct MyMessage {
    value: i32,
    string: String,
}

fn main() {
    App::new()
        .add_plugin(LogPlugin::default())
        .add_plugins(MinimalPlugins)
        .add_event::<MyMessage>()
        .add_plugin(WasmPlugin::<MyMessage>::new(vec![SCRIPTS_WASM.into()]))
        .add_startup_system(hello_world_system)
        .add_system(listen_for_message)
        .run();
}

fn hello_world_system() {
    info!("Hello, world from outside wasm!");
}

fn listen_for_message(mut events: EventReader<MyMessage>) {
    for event in events.iter() {
        info!("Got message from wasm: {:?}", event);
    }
}
