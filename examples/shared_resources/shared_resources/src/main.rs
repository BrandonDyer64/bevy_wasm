use bevy::prelude::*;
use bevy_wasm::prelude::*;
use shared_resources_protocol::{HostMessage, ModMessage, MyCoolResource, PROTOCOL_VERSION};

pub static MOD_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shared_resources_mod.wasm"));

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(MyCoolResource {
            value: 0,
            v2: 1234567890,
        })
        .add_plugin(
            WasmPlugin::<HostMessage, ModMessage>::new(PROTOCOL_VERSION)
                .share_resource::<MyCoolResource>(),
        )
        .add_startup_system(insert_mods)
        .add_startup_system(setup)
        .add_system(update_resource)
        .run();
}

fn insert_mods(mut wasm: ResMut<WasmResource<HostMessage, ModMessage>>) {
    wasm.insert_wasm(MOD_WASM);
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 3.5, 5.0)
            .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        ..default()
    });
}

fn update_resource(mut my_cool_resource: ResMut<MyCoolResource>) {
    my_cool_resource.value += 1;
    info!("{:?}", my_cool_resource);
}
