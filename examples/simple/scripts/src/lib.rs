use bevy_wasm_sys::{info, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct MyMessage {
    value: i32,
    string: String,
}

#[no_mangle]
pub unsafe extern "C" fn build_app() {
    info!("Hello from build_app inside wasm!");
    App::new()
        .add_plugin(FFIPlugin::<MyMessage>::default())
        .add_startup_system(startup_system)
        .add_system(update_resource)
        .add_system(send_a_message)
        .run();
}

#[derive(Resource)]
struct MyResource {
    value: i32,
}

fn startup_system(mut commands: Commands) {
    info!("Hello from startup_system inside wasm!");
    commands.insert_resource(MyResource { value: 0 });
}

fn update_resource(mut resource: ResMut<MyResource>) {
    resource.value += 1;
}

fn send_a_message(mut events: EventWriter<MyMessage>, resource: Res<MyResource>) {
    info!("Sending a message");
    events.send(MyMessage {
        value: resource.value,
        string: "Hello from send_a_message!".to_string(),
    });
}
