use bevy::{log::LogPlugin, prelude::*};
use bevy_wasm::*;
use simple_protocol::{HostMessage, ModMessage};

pub static SCRIPTS_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/scripts.wasm"));

fn main() {
    App::new()
        .add_plugin(LogPlugin::default())
        .add_plugins(MinimalPlugins)
        .add_event::<HostMessage>()
        .add_event::<ModMessage>()
        .add_plugin(WasmPlugin::<HostMessage, ModMessage>::new(vec![
            SCRIPTS_WASM.into(),
        ]))
        .add_startup_system(hello_world_system)
        .add_system(send_a_message)
        .add_system(listen_for_message)
        .run();
}

fn hello_world_system() {
    info!("Hello, world from host!");
}

fn send_a_message(mut events: EventWriter<HostMessage>) {
    events.send(HostMessage::SaySomething("Hello from host!".to_string()));
}

fn listen_for_message(mut events: EventReader<ModMessage>) {
    for event in events.iter() {
        info!("Got message from wasm: {:?}", event);
    }
}
