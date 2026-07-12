use bevy_wasm_sys::prelude::*;
use simple_protocol::{GameMessage, ModMessage, PROTOCOL_VERSION};
use bevy_ecs::message::{MessageWriter, MessageReader};
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn build_app() {
    App::new()
        .add_plugins(FFIPlugin::<GameMessage, ModMessage>::new(PROTOCOL_VERSION))
        .add_systems(Update, listen_for_game_messages)
        .add_systems(Update, send_messages_to_game)
        .run();
}

fn listen_for_game_messages(mut events: MessageReader<GameMessage>) {
    for event in events.read() {
        match event {
            GameMessage::HiThere => {
                info!("The game said hi there!");
            }
        }
    }
}

fn send_messages_to_game(mut events: MessageWriter<ModMessage>) {
    events.write(ModMessage::Hello);
}
