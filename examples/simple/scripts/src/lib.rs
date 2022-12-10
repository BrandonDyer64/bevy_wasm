use bevy_wasm_sys::{info, prelude::*};
use simple_protocol::{HostMessage, ModMessage};

#[no_mangle]
pub unsafe extern "C" fn build_app() {
    info!("Hello from build_app inside wasm!");
    App::new()
        .add_plugin(FFIPlugin::<HostMessage, ModMessage>::new())
        .add_startup_system(startup_system)
        .add_system(update_resource)
        .add_system(send_a_message)
        .add_system(listen_for_message)
        .run();
}

#[derive(Resource)]
struct MyResource {
    value: i32,
}

fn startup_system(mut commands: Commands) {
    info!("Hello from startup_system inside mod!");
    commands.insert_resource(MyResource { value: 0 });
}

fn update_resource(mut resource: ResMut<MyResource>) {
    resource.value += 1;
}

fn send_a_message(mut events: EventWriter<ModMessage>, resource: Res<MyResource>) {
    events.send(ModMessage::SaySomething(format!(
        "Hello from wasm! resource value = {}",
        resource.value
    )));
}

fn listen_for_message(mut events: EventReader<HostMessage>) {
    for event in events.iter() {
        info!("Got message from host: {:?}", event);
    }
}
