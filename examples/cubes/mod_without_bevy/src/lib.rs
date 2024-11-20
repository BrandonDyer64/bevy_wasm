use bevy_wasm_sys_core::events::{get_next_event, send_event};
use bevy_wasm_sys_core::{ffi, info};
use cubes_protocol::{HostMessage, ModMessage};

use std::ffi::c_void;
use std::time::Duration;

const MOD_STATE: u64 = 0xf6a11546; // Should be unique to each mod

#[derive(Debug)]
struct AppState {
    entity_id: Option<u32>,
    x: f32,
    y: f32,
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn build_app() {
    info!("Hello from build_app inside mod_without_bevy!");
    let app_state = AppState {
        entity_id: None,
        x: 0.0,
        y: 0.0,
    };

    send_event(&ModMessage::SpawnCube {
        mod_state: MOD_STATE,
        color: (1.0, 0.0, 0.0),
    });

    let app = Box::new(app_state);
    let app_ptr = Box::into_raw(app);
    let app_ptr = app_ptr as *const c_void;

    ffi::store_app(app_ptr);
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn update(app_state: *mut c_void) {
    let app_state = app_state as *mut AppState;
    update_app_state(&mut *app_state);
}

fn update_app_state(app_state: &mut AppState) {
    let time_since_start = unsafe { Duration::from_nanos(ffi::get_time_since_startup()) };

    let time: f32 = time_since_start.as_secs_f32() + std::f32::consts::PI;

    app_state.y = time.sin() + 1.5;
    app_state.x = -time.cos();

    while let Some(event) = get_next_event::<HostMessage>() {
        if let HostMessage::SpawnedCube {
            entity_id,
            mod_state: MOD_STATE, // Must be for us
        } = event
        {
            app_state.entity_id = Some(entity_id);
        }
    }

    let Some(entity_id) = app_state.entity_id else { return };

    send_event(&ModMessage::MoveCube {
        entity_id,
        x: app_state.x,
        y: app_state.y,
        z: 0.0,
    });
}
