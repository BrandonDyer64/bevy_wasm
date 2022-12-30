use bevy_wasm_sys::{ecs::extern_res::ExternResources, prelude::*};
use shared_resources_protocol::{HostMessage, ModMessage, MyCoolResource, PROTOCOL_VERSION};

#[no_mangle]
pub unsafe extern "C" fn build_app() {
    info!("Hello from build_app inside mod_with_bevy!");
    App::new()
        .add_plugin(FFIPlugin::<HostMessage, ModMessage>::new(PROTOCOL_VERSION))
        .add_startup_system(startup_system)
        .add_system(print_resource_value)
        .run();
}

fn startup_system(mut resources: ResMut<ExternResources>) {
    info!("Hello from startup_system inside mod!");
    warn!("This is a warning!");
    error!("This is an error!");
    resources.insert::<MyCoolResource>();
}

fn print_resource_value(resource: ExternRes<MyCoolResource>) {
    info!("{:?}", resource);
}
