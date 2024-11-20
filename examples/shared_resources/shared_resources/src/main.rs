use bevy::{log::LogPlugin, prelude::*};
use bevy_wasm::prelude::*;
use shared_resources_protocol::{HostMessage, ModMessage, MyCoolResource, PROTOCOL_VERSION};

fn main() {
    App::new()
        .add_plugins(AssetPlugin::default())
        .add_plugins(LogPlugin::default())
        .add_plugins(MinimalPlugins)
        .insert_resource(MyCoolResource {
            value: 0,
            string: "Hello from MyCoolResource!".to_string(),
        })
        .add_plugins(
            WasmPlugin::<HostMessage, ModMessage>::new(PROTOCOL_VERSION)
                .share_resource::<MyCoolResource>(),
        )
        .add_systems(Startup, insert_mods)
        .add_systems(Update, update_resource)
        .run();
}

fn insert_mods(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(WasmMod {
        wasm: asset_server.load("shared_resources_mod.wasm"),
    });
}

fn update_resource(mut my_cool_resource: ResMut<MyCoolResource>) {
    my_cool_resource.value += 1;
}
