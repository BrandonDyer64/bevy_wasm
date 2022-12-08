use std::ffi::c_void;

use bevy_app::{App, Plugin};

use crate::ffi::store_app;

pub struct FFIPlugin;

impl Plugin for FFIPlugin {
    fn build(&self, app: &mut App) {
        app.set_runner(app_runner);
    }
}

fn app_runner(mut app: App) {
    app.update();
    let app = Box::new(app);
    let app_ptr = Box::into_raw(app);
    let app_ptr = app_ptr as *const c_void;
    unsafe { store_app(app_ptr) };
}
