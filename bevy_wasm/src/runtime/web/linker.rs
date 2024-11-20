use std::sync::{Arc, RwLock};

use bevy::prelude::{error, info, warn};
use bevy_wasm_shared::version::Version;
use colored::*;
use js_sys::{Object, Reflect, Uint8Array, WebAssembly};
use wasm_bindgen::{
    closure::{IntoWasmClosure, WasmClosure},
    prelude::{Closure, JsValue},
};
use uuid::Uuid;

use crate::mod_state::ModState;

fn link<T>(target: &JsValue, name: &str, closure: impl IntoWasmClosure<T> + 'static)
where
    T: WasmClosure + ?Sized,
{
    let closure = Closure::new(closure);
    Reflect::set(target, &JsValue::from_str(name), closure.as_ref()).unwrap();
    Box::leak(Box::new(closure)); // TODO: Don't just leak the closures.
}

#[allow(clippy::redundant_clone)]
pub fn build_linker(
    protocol_version: Version,
    mod_state: Arc<RwLock<ModState>>,
    memory: Arc<RwLock<Option<WebAssembly::Memory>>>,
) -> Object {
    let host = Object::new();

    link::<dyn FnMut(i32, u32)>(&host, "console_info", {
        let memory = memory.clone();
        move |ptr, len| {
            if let Some(memory) = memory.read().unwrap().as_ref() {
                let buffer = Uint8Array::new(&memory.buffer())
                    .slice(ptr as u32, ptr as u32 + len)
                    .to_vec();
                let text = std::str::from_utf8(&buffer).unwrap();
                info!("MOD: {}", text);
            }
        }
    });

    link::<dyn FnMut(i32, u32)>(&host, "console_warn", {
        let memory = memory.clone();
        move |ptr, len| {
            if let Some(memory) = memory.read().unwrap().as_ref() {
                let buffer = Uint8Array::new(&memory.buffer())
                    .slice(ptr as u32, ptr as u32 + len)
                    .to_vec();
                let text = std::str::from_utf8(&buffer).unwrap();
                warn!("MOD: {}", text);
            }
        }
    });

    link::<dyn FnMut(i32, u32)>(&host, "console_error", {
        let memory = memory.clone();
        move |ptr, len| {
            if let Some(memory) = memory.read().unwrap().as_ref() {
                let buffer = Uint8Array::new(&memory.buffer())
                    .slice(ptr as u32, ptr as u32 + len)
                    .to_vec();
                let text = std::str::from_utf8(&buffer).unwrap();
                error!("MOD: {}", text);
            }
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
        let memory = memory.clone();
        move |ptr: i32, len: u32| -> u32 {
            let next_event = mod_state.write().unwrap().events_in.pop_front();
            if let Some(next_event) = next_event {
                if next_event.len() > len as usize {
                    error!("Serialized event is too long");
                    return 0;
                }
                let arr = Uint8Array::from(&next_event[..]);
                if let Some(memory) = memory.read().unwrap().as_ref() {
                    Uint8Array::new(&memory.buffer()).set(&arr, ptr as u32);
                    next_event.len() as u32
                } else {
                    0
                }
            } else {
                0
            }
        }
    });

    link::<dyn FnMut(i32, u32)>(&host, "send_serialized_event", {
        let mod_state = mod_state.clone();
        let memory = memory.clone();
        move |ptr, len| {
            if let Some(memory) = memory.read().unwrap().as_ref() {
                let buffer = Uint8Array::new(&memory.buffer())
                    .slice(ptr as u32, ptr as u32 + len)
                    .to_vec();
                mod_state.write().unwrap().events_out.push(buffer.into());
            }
        }
    });

    link::<dyn FnMut() -> u64>(&host, "get_protocol_version", {
        move || -> u64 { protocol_version.to_u64() }
    });

    link::<dyn FnMut(u64, u64, i32, u32) -> u32>(&host, "get_resource", {
        let mod_state = mod_state.clone();
        let memory = memory.clone();
        move |uuid_0, uuid_1, buffer_ptr, buffer_len| -> u32 {
            let uuid = Uuid::from_u64_pair(uuid_0, uuid_1);
            let resource_bytes = mod_state
                .write()
                .unwrap()
                .shared_resource_values
                .remove(&uuid);
            let Some(resource_bytes) = resource_bytes else { return 0 };
            if resource_bytes.len() > buffer_len as usize {
                error!("Serialized event is too long");
                return 0;
            }
            let arr = Uint8Array::from(&resource_bytes[..]);
            if let Some(memory) = memory.read().unwrap().as_ref() {
                Uint8Array::new(&memory.buffer()).set(&arr, buffer_ptr as u32);
                resource_bytes.len() as u32
            } else {
                0
            }
        }
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
