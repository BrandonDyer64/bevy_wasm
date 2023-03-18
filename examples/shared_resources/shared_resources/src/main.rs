use bevy::{log::LogPlugin, prelude::*};
use bevy_wasm::prelude::*;
use shared_resources_protocol::{HostMessage, ModMessage, MyCoolResource, PROTOCOL_VERSION};

pub static MOD_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shared_resources_mod.wasm"));

fn main() {
    App::new()
        .add_plugin(LogPlugin::default())
        .add_plugins(MinimalPlugins)
        .insert_resource(MyCoolResource {
            value: 0,
            string: "Hello from MyCoolResource!".to_string(),
        })
        .add_plugin(
            WasmPlugin::<HostMessage, ModMessage>::new(PROTOCOL_VERSION)
                .share_resource::<MyCoolResource>(),
        )
        .add_startup_system(insert_mods)
        .add_system(update_resource)
        .run();
}

fn insert_mods(mut commands: Commands, wasm_engine: Res<WasmEngine>) {
    commands.spawn(WasmModInternal::new(&wasm_engine, MOD_WASM).unwrap());
}

fn update_resource(mut my_cool_resource: ResMut<MyCoolResource>) {
    my_cool_resource.value += 1;
}
