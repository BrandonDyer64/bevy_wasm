use bevy_wasm_sys::{ecs::extern_res::ExternResources, prelude::*};
use shared_resources_protocol::{HostMessage, ModMessage, MyCoolResource, PROTOCOL_VERSION};

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn build_app() {
    info!("Hello from build_app inside mod_with_bevy!");
    App::new()
        .add_plugins(FFIPlugin::<HostMessage, ModMessage>::new(PROTOCOL_VERSION))
        .add_systems(Startup, startup_system)
        .add_systems(Update, print_resource_value)
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
