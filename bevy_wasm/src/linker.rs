use std::time::Instant;

use bevy_log::prelude::*;
use bevy_wasm_shared::prelude::*;
use colored::*;
use wasmtime::*;

use crate::{resource::State, Message};

pub(crate) fn build_linker<In: Message, Out: Message>(
    engine: &Engine,
    protocol_version: Version,
) -> Linker<State<In, Out>> {
    let mut linker: Linker<State<In, Out>> = Linker::new(&engine);

    linker
        .func_wrap(
            "host",
            "console_info",
            |mut caller: Caller<'_, State<In, Out>>, msg: i32, len: u32| {
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => panic!("failed to find host memory"),
                };

                let data = mem
                    .data(&caller)
                    .get(msg as u32 as usize..)
                    .and_then(|arr| arr.get(..len as u32 as usize))
                    .unwrap();
                let string = std::str::from_utf8(data).unwrap();
                info!(target: "MOD", "{}", string);
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "host",
            "console_warn",
            |mut caller: Caller<'_, State<In, Out>>, msg: i32, len: u32| {
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => panic!("failed to find host memory"),
                };

                let data = mem
                    .data(&caller)
                    .get(msg as u32 as usize..)
                    .and_then(|arr| arr.get(..len as u32 as usize))
                    .unwrap();
                let string = std::str::from_utf8(data).unwrap();
                warn!(target: "MOD", "{}", string);
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "host",
            "console_error",
            |mut caller: Caller<'_, State<In, Out>>, msg: i32, len: u32| {
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => panic!("failed to find host memory"),
                };

                let data = mem
                    .data(&caller)
                    .get(msg as u32 as usize..)
                    .and_then(|arr| arr.get(..len as u32 as usize))
                    .unwrap();
                let string = std::str::from_utf8(data).unwrap();
                error!(target: "MOD", "{}", string);
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "host",
            "store_app",
            |mut caller: Caller<'_, State<In, Out>>, app_ptr: i32| {
                caller.data_mut().app_ptr = app_ptr;
                info!("{} 0x{:X}", "Storing app pointer:".italic(), app_ptr);
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "host",
            "send_serialized_event",
            |mut caller: Caller<'_, State<In, Out>>, msg: i32, len: u32| {
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => panic!("failed to find host memory"),
                };

                let data = mem
                    .data(&caller)
                    .get(msg as u32 as usize..)
                    .and_then(|arr| arr.get(..len as u32 as usize))
                    .unwrap();
                let event: Out = match bincode::deserialize(data) {
                    Ok(event) => event,
                    Err(e) => {
                        error!("Failed to deserialize event from mod: {}", e);
                        return;
                    }
                };
                caller.data_mut().events_out.push(event);
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "host",
            "get_next_event",
            |mut caller: Caller<'_, State<In, Out>>, arena: i32, len: u32| -> u32 {
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => panic!("failed to find host memory"),
                };

                let Some(event) = caller.data_mut().events_in.pop_front() else { return 0 };

                let serialized_event = match bincode::serialize(&event) {
                    Ok(event) => event,
                    Err(e) => {
                        error!("Failed to serialize event: {}", e);
                        return 0;
                    }
                };

                let buffer = mem
                    .data_mut(&mut caller)
                    .get_mut(arena as u32 as usize..)
                    .and_then(|arr| arr.get_mut(..len as u32 as usize))
                    .unwrap();

                buffer[..serialized_event.len()].copy_from_slice(serialized_event.as_slice());
                serialized_event.len() as u32
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "host",
            "get_time_since_startup",
            |caller: Caller<'_, State<In, Out>>| -> u64 {
                let startup_time = caller.data().startup_time;
                let delta = Instant::now() - startup_time;
                delta.as_nanos() as u64
            },
        )
        .unwrap();
    linker
        .func_wrap("host", "get_protocol_version", move || -> u64 {
            protocol_version.to_u64()
        })
        .unwrap();

    // Because bevy wants to use wasm-bindgen
    linker
        .func_wrap(
            "__wbindgen_placeholder__",
            "__wbindgen_describe",
            |v: i32| {
                info!("__wbindgen_describe: {}", v);
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "__wbindgen_placeholder__",
            "__wbindgen_throw",
            |mut caller: Caller<'_, State<In, Out>>, msg: i32, len: i32| {
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => panic!("failed to find host memory"),
                };

                let data = mem
                    .data(&caller)
                    .get(msg as u32 as usize..)
                    .and_then(|arr| arr.get(..len as u32 as usize))
                    .unwrap();
                let string = std::str::from_utf8(data).unwrap();
                info!("{}", string);
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "__wbindgen_externref_xform__",
            "__wbindgen_externref_table_grow",
            |v: i32| -> i32 {
                info!("__wbindgen_externref_table_grow: {}", v);
                0
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "__wbindgen_externref_xform__",
            "__wbindgen_externref_table_set_null",
            |v: i32| {
                info!("__wbindgen_externref_table_set_null: {}", v);
            },
        )
        .unwrap();
    linker
}
