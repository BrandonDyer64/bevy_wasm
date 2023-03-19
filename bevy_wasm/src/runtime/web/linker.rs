use std::sync::{Arc, RwLock};

use bevy::prelude::info;
use bevy_wasm_shared::version::Version;
use colored::*;
use js_sys::{Object, Reflect, WebAssembly};
use wasm_bindgen::{
    closure::{IntoWasmClosure, WasmClosure},
    prelude::{Closure, JsValue},
};

use crate::mod_state::ModState;

fn link<T>(target: &JsValue, name: &str, closure: impl IntoWasmClosure<T> + 'static)
where
    T: WasmClosure + ?Sized,
{
    let closure = Closure::new(closure);
    Reflect::set(target, &JsValue::from_str(name), closure.as_ref()).unwrap();
    Box::leak(Box::new(closure));
}

pub fn build_linker(
    protocol_version: Version,
    mod_state: Arc<RwLock<ModState>>,
    memory: Arc<RwLock<Option<WebAssembly::Memory>>>,
) -> Object {
    let host = Object::new();

    link::<dyn FnMut(i32, u32)>(&host, "console_info", {
        let _memory = memory.clone();
        move |ptr, len| {
            info!("CONSOLE INFO {} {}", ptr, len);
        }
    });

    link::<dyn FnMut(i32, u32)>(&host, "console_warn", {
        let _memory = memory.clone();
        move |ptr, len| {
            info!("CONSOLE WARN {} {}", ptr, len);
        }
    });

    link::<dyn FnMut(i32, u32)>(&host, "console_error", {
        let _memory = memory.clone();
        move |ptr, len| {
            info!("CONSOLE ERROR {} {}", ptr, len);
        }
    });

    link::<dyn FnMut(i32)>(&host, "store_app", {
        let mod_state = mod_state.clone();
        move |ptr| {
            mod_state.write().unwrap().app_ptr = ptr;
            info!("{} 0x{:X}", "Storing app pointer:".italic(), ptr);
        }
    });

    link::<dyn FnMut() -> u64>(&host, "get_time_since_startup", {
        let mod_state = mod_state.clone();
        move || -> u64 { mod_state.read().unwrap().startup_time.elapsed().as_nanos() as u64 }
    });

    link::<dyn FnMut(i32, u32) -> u32>(&host, "get_next_event", {
        let mod_state = mod_state.clone();
        move |ptr, len| -> u32 { 0 }
    });

    link::<dyn FnMut(i32, u32)>(&host, "send_serialized_event", {
        let mod_state = mod_state.clone();
        let memory = memory.clone();
        move |ptr, len| {}
    });

    link::<dyn FnMut() -> u64>(&host, "get_protocol_version", {
        move || -> u64 { protocol_version.to_u64() }
    });

    link::<dyn FnMut(u64, u64, i32, u32) -> u32>(&host, "get_resource", {
        let mod_state = mod_state.clone();
        let memory = memory.clone();
        move |uuid_0, uuid_1, buffer_ptr, buffer_len| -> u32 { 0 }
    });

    // __wbindgen_placeholder__
    let wbp = Object::new();

    link::<dyn FnMut(i32)>(&wbp, "__wbindgen_describe", {
        move |v| {
            info!("__wbindgen_describe: {}", v);
        }
    });

    link::<dyn FnMut(i32, i32)>(&wbp, "__wbindgen_throw", {
        move |msg, len| {
            info!("__wbindgen_throw: {} {}", msg, len);
        }
    });

    // __wbindgen_externref_xform__
    let wbxf = Object::new();

    link::<dyn FnMut(i32) -> i32>(&wbxf, "__wbindgen_externref_table_grow", {
        move |v| -> i32 {
            info!("__wbindgen_externref_table_grow: {}", v);
            0
        }
    });

    link::<dyn FnMut(i32)>(&wbxf, "__wbindgen_externref_table_set_null", {
        move |v| {
            info!("__wbindgen_externref_table_set_null: {}", v);
        }
    });

    let imports = Object::new();
    Reflect::set(
        &imports,
        &JsValue::from_str("__wbindgen_placeholder__"),
        &wbp,
    )
    .unwrap();
    Reflect::set(
        &imports,
        &JsValue::from_str("__wbindgen_externref_xform__"),
        &wbxf,
    )
    .unwrap();
    Reflect::set(&imports, &JsValue::from_str("host"), &host).unwrap();
    imports
}
