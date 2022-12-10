use bevy_wasm_sys::{info, prelude::*};
use cubes_protocol::{HostMessage, ModMessage};

const MOD_STATE: u64 = 0xa6e79eb9; // Should be unique to each mod

#[no_mangle]
pub unsafe extern "C" fn build_app() {
    info!("Hello from build_app inside mod_with_bevy!");
    App::new()
        .add_plugin(FFIPlugin::<HostMessage, ModMessage>::new())
        .add_startup_system(startup_system)
        .add_system(update_cube)
        .add_system(listen_for_message)
        .run();
}

#[derive(Resource)]
struct CubePosition {
    entity_id: Option<u32>,
    x: f32,
    y: f32,
    z: f32,
}

fn startup_system(mut commands: Commands, mut events: EventWriter<ModMessage>) {
    info!("Hello from startup_system inside mod!");
    commands.insert_resource(CubePosition {
        entity_id: None,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
    events.send(ModMessage::SpawnCube {
        mod_state: MOD_STATE,
        color: (0.0, 1.0, 0.0),
    });
}

fn update_cube(
    mut resource: ResMut<CubePosition>,
    time: Res<Time>,
    mut events: EventWriter<ModMessage>,
) {
    // Move the cube up
    resource.y = time.elapsed_seconds().sin() + 1.5;

    // Ensure the cube has been spawned on the host
    let entity_id = match resource.entity_id {
        Some(entity_id) => entity_id,
        None => return,
    };

    // Tell the game we moved the cube
    events.send(ModMessage::MoveCube {
        entity_id,
        x: resource.x,
        y: resource.y,
        z: resource.z,
    });
}

fn listen_for_message(mut events: EventReader<HostMessage>, mut resource: ResMut<CubePosition>) {
    for event in events.iter() {
        match event {
            HostMessage::SpawnedCube {
                entity_id,
                mod_state: MOD_STATE, // Must be for us
            } => {
                resource.entity_id = Some(*entity_id);
            }
            _ => {}
        }
    }
}
