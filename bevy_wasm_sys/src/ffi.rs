use std::ffi::c_void;

use bevy_app::App;

#[link(wasm_import_module = "host")]
extern "C" {
    pub fn ping(v: i32) -> i32;
    pub fn store_app(app: *const c_void);
    pub fn console_info(msg: *const u8, len: usize);
    pub fn console_warn(msg: *const u8, len: usize);
    pub fn console_error(msg: *const u8, len: usize);
    pub fn send_serialized_event(event: *const u8, len: usize);
    // pub fn get_next_event(event: *const u8, len: usize) -> i32;
}

#[cfg(feature = "bevy")]
#[no_mangle]
pub unsafe extern "C" fn update(app: *mut c_void) {
    let app = app as *mut App;
    (*app).update();
}
