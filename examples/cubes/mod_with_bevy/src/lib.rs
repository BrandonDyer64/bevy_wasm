use bevy_wasm_sys::prelude::*;
use cubes_protocol::{HostMessage, ModMessage, PROTOCOL_VERSION};

const MOD_STATE: u64 = 0xa6e79eb9; // Should be unique to each mod

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn build_app() {
    info!("Hello from build_app inside mod_with_bevy!");
    App::new()
        .add_plugins(FFIPlugin::<HostMessage, ModMessage>::new(PROTOCOL_VERSION))
        .add_systems(Startup, startup_system)
        .add_systems(Update, (update_cube, listen_for_message))
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
    warn!("This is a warning!");
    error!("This is an error!");
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
    let time: f32 = time.elapsed_seconds();
    // Move the cube in a circle
    resource.y = time.sin() + 1.5;
    resource.z = time.cos();

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
    for event in events.read() {
        if let HostMessage::SpawnedCube {
            entity_id,
            mod_state: MOD_STATE, // Must be for us
        } = event
        {
            resource.entity_id = Some(*entity_id);
        }
    }
}
