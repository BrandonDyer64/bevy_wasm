use bevy_wasm_sys::prelude::*;
use simple_protocol::{GameMessage, ModMessage};

#[no_mangle]
pub unsafe extern "C" fn build_app() {
    App::new()
        .add_plugin(FFIPlugin::<GameMessage, ModMessage>::new())
        .add_system(listen_for_game_messages)
        .add_system(send_messages_to_game)
        .run();
}

fn listen_for_game_messages(mut events: EventReader<GameMessage>) {
    for event in events.iter() {
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
