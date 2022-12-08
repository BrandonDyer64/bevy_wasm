use bevy_wasm_sys::{info, prelude::*};

#[no_mangle]
pub unsafe extern "C" fn build_app() {
    let msg = "Hello from build_app!";
    bevy_wasm_sys::ffi::console_info(msg.as_ptr(), msg.len());
    App::new()
        .add_plugin(FFIPlugin)
        .add_startup_system(startup_system)
        .add_system(update_resource)
        .run();
}

#[derive(Resource)]
struct MyResource {
    value: i32,
}

fn startup_system(mut commands: Commands) {
    info!("Hello from startup_system");
    commands.insert_resource(MyResource { value: 0 });
}

fn update_resource(mut resource: ResMut<MyResource>) {
    resource.value += 1;
    info!("Hello from update_resource! value = {}", resource.value);
}
