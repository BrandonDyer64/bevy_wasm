use bevy_wasm_sys::prelude::*;
use simple_protocol::{GameMessage, ModMessage, PROTOCOL_VERSION};

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn build_app() {
    App::new()
        .add_plugins(FFIPlugin::<GameMessage, ModMessage>::new(PROTOCOL_VERSION))
        .add_systems(Update, listen_for_game_messages)
        .add_systems(Update, send_messages_to_game)
        .run();
}

fn listen_for_game_messages(mut events: EventReader<GameMessage>) {
    for event in events.read() {
        match event {
            GameMessage::HiThere => {
                info!("The game said hi there!");
            }
        }
    }
}

fn send_messages_to_game(mut events: EventWriter<ModMessage>) {
    events.send(ModMessage::Hello);
}
