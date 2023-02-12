use bevy::{log::LogPlugin, prelude::*};
use bevy_wasm::prelude::*;
use simple_protocol::{GameMessage, ModMessage, PROTOCOL_VERSION};

static MOD_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/simple_mod.wasm"));

fn main() {
    App::new()
        .add_plugin(LogPlugin::default())
        .add_plugins(MinimalPlugins)
        .add_plugin(WasmPlugin::<GameMessage, ModMessage>::new(PROTOCOL_VERSION))
        .add_startup_system(insert_mods)
        .add_system(listen_for_mod_messages)
        .add_system(send_messages_to_mods)
        .run();
}

fn insert_mods(mut commands: Commands, wasm_engine: Res<WasmEngine>) {
    commands.spawn(WasmMod::new(&wasm_engine, MOD_WASM).unwrap());
}

fn listen_for_mod_messages(mut events: EventReader<ModMessage>) {
    for event in events.iter() {
        match event {
            ModMessage::Hello => {
                info!("The mod said hello!");
            }
        }
    }
}

fn send_messages_to_mods(mut events: EventWriter<GameMessage>) {
    events.send(GameMessage::HiThere);
}
