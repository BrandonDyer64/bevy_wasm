use std::time::Instant;

use anyhow::Result;
use bevy::prelude::*;
use bevy_wasm_shared::prelude::*;
use colored::*;
use uuid::Uuid;
use wasmtime::*;

use super::WasiModState;

pub(crate) fn build_linker(
    engine: &Engine,
    protocol_version: Version,
) -> Result<Linker<WasiModState>> {
    let mut linker: Linker<WasiModState> = Linker::new(engine);
    wasi_common::sync::add_to_linker(&mut linker, |s| &mut s.wasi_ctx)?;

    linker.func_wrap(
        "host",
        "console_info",
        |mut caller: Caller<WasiModState>, msg: i32, len: u32| {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => panic!("failed to find mod memory"),
            };

            let Some(data) = mem
                .data(&caller)
                .get(msg as u32 as usize..)
                .and_then(|arr| arr.get(..len as usize)) else {
                    error!("Failed to get data from memory");
                    return;
                };

            // SAFETY: We know that the memory is valid UTF-8 because it was written from a string in the mod
            let string = unsafe { std::str::from_utf8_unchecked(data) };
            info!(target: "MOD", "{}", string);
        },
    )?;
    linker.func_wrap(
        "host",
        "console_warn",
        |mut caller: Caller<WasiModState>, msg: i32, len: u32| {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => panic!("failed to find mod memory"),
            };

            let Some(data) = mem
                .data(&caller)
                .get(msg as u32 as usize..)
                .and_then(|arr| arr.get(..len as usize)) else {
                    error!("Failed to get data from memory");
                    return;
                };

            // SAFETY: We know that the memory is valid UTF-8 because it was written from a string in the mod
            let string = unsafe { std::str::from_utf8_unchecked(data) };
            warn!(target: "MOD", "{}", string);
        },
    )?;
    linker.func_wrap(
        "host",
        "console_error",
        |mut caller: Caller<WasiModState>, msg: i32, len: u32| {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => panic!("failed to find mod memory"),
            };

            let Some(data) = mem
                .data(&caller)
                .get(msg as u32 as usize..)
                .and_then(|arr| arr.get(..len as usize)) else {
                    error!("Failed to get data from memory");
                    return;
                };

            // SAFETY: We know that the memory is valid UTF-8 because it was written from a string in the mod
            let string = unsafe { std::str::from_utf8_unchecked(data) };
            error!(target: "MOD", "{}", string);
        },
    )?;
    linker.func_wrap(
        "host",
        "store_app",
        |mut caller: Caller<WasiModState>, app_ptr: i32| {
            caller.data_mut().mod_state.app_ptr = app_ptr;
            info!("{} 0x{:X}", "Storing app pointer:".italic(), app_ptr);
        },
    )?;
    linker.func_wrap(
        "host",
        "send_serialized_event",
        |mut caller: Caller<WasiModState>, msg: i32, len: u32| {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => panic!("failed to find mod memory"),
            };

            let Some(data) = mem
                .data(&caller)
                .get(msg as u32 as usize..)
                .and_then(|arr| arr.get(..len as usize))
                .map(|x| x.into()) else {
                    error!("Failed to get data from memory");
                    return;
                };

            caller.data_mut().mod_state.events_out.push(data);
        },
    )?;
    linker.func_wrap(
        "host",
        "get_next_event",
        |mut caller: Caller<WasiModState>, arena: i32, len: u32| -> u32 {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => panic!("failed to find mod memory"),
            };

            let Some(serialized_event) = caller.data_mut().mod_state.events_in.pop_front() else { return 0; };

            let Some(buffer) = mem
                .data_mut(&mut caller)
                .get_mut(arena as u32 as usize..)
                .and_then(|arr| arr.get_mut(..len as usize)) else {
                    error!("Failed to get data from memory");
                    return 0;
                };

            buffer[..serialized_event.len()].copy_from_slice(&serialized_event);
            serialized_event.len() as u32
        },
    )?;
    linker.func_wrap(
        "host",
        "get_resource",
        |mut caller: Caller<WasiModState>,
         uuid_0: u64,
         uuid_1: u64,
         buffer: i32,
         buffer_len: u32|
         -> u32 {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => panic!("failed to find mod memory"),
            };

            let uuid = Uuid::from_u64_pair(uuid_0, uuid_1);
            let resource_bytes = caller.data_mut().mod_state.shared_resource_values.remove(&uuid);

            let resource_bytes = match resource_bytes {
                Some(resource_bytes) => resource_bytes,
                None => return 0,
            };

            let Some(buffer) = mem
                .data_mut(&mut caller)
                .get_mut(buffer as u32 as usize..)
                .and_then(|arr| arr.get_mut(..buffer_len as usize)) else {
                    error!("Failed to get data from memory");
                    return 0;
                };

            buffer[..resource_bytes.len()].copy_from_slice(&resource_bytes);
            resource_bytes.len() as u32
        },
    )?;
    linker.func_wrap(
        "host",
        "get_time_since_startup",
        |caller: Caller<WasiModState>| -> u64 {
            let startup_time = caller.data().mod_state.startup_time;
            let delta = Instant::now() - startup_time;
            delta.as_nanos() as u64
        },
    )?;
    linker.func_wrap("host", "get_protocol_version", move || -> u64 {
        protocol_version.to_u64()
    })?;

    // Because bevy wants to use wasm-bindgen
    linker.func_wrap(
        "__wbindgen_placeholder__",
        "__wbindgen_describe",
        |v: i32| {
            info!("__wbindgen_describe: {}", v);
        },
    )?;
    linker.func_wrap(
        "__wbindgen_placeholder__",
        "__wbindgen_throw",
        |mut caller: Caller<WasiModState>, msg: i32, len: i32| {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => panic!("failed to find mod memory"),
            };

            let Some(data) = mem
                .data(&caller)
                .get(msg as u32 as usize..)
                .and_then(|arr| arr.get(..len as usize)) else {
                    error!("Failed to get data from memory");
                    return;
                };

            // SAFETY: We know that the memory is valid UTF-8 because it was written from a string in the mod
            let string = unsafe { std::str::from_utf8_unchecked(data) };
            info!("{}", string);
        },
    )?;
    linker.func_wrap(
        "__wbindgen_externref_xform__",
        "__wbindgen_externref_table_grow",
        |v: i32| -> i32 {
            info!("__wbindgen_externref_table_grow: {}", v);
            0
        },
    )?;
    linker.func_wrap(
        "__wbindgen_externref_xform__",
        "__wbindgen_externref_table_set_null",
        |v: i32| {
            info!("__wbindgen_externref_table_set_null: {}", v);
        },
    )?;
    Ok(linker)
}
