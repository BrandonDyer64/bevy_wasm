//! Mod Bevy games with WebAssembly
//!
//! See [examples/cubes](https://github.com/BrandonDyer64/bevy_wasm/tree/main/examples/cubes)
//! for a comprehensive example of how to use this.
//!
//! For building mods, see the sister crate [bevy_wasm_sys](https://docs.rs/bevy_wasm_sys).

#![deny(missing_docs)]

use std::{collections::VecDeque, time::Instant};

use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::{
    prelude::{EventReader, EventWriter},
    system::{ResMut, Resource},
};
use bevy_log::{error, info, warn};
use bevy_wasm_shared::prelude::*;
use colored::*;
use serde::{de::DeserializeOwned, Serialize};
use wasmtime::*;

/// Any data type that can be used as a Host <-> Mod message
///
/// Must be Clone, Send, and Sync, and must be (de)serializable with serde.
///
/// `bevy_wasm` uses `bincode` for serialization, so it's relatively fast.
pub trait Message: Send + Sync + Serialize + DeserializeOwned + Clone + 'static {}

impl<T> Message for T where T: Send + Sync + Serialize + DeserializeOwned + Clone + 'static {}

/// Internal mod state
pub struct State<In: Message, Out: Message> {
    /// Time when the mod was loaded
    pub startup_time: Instant,

    /// Pointer given to us in `store_app`
    pub app_ptr: i32,

    /// Events that have been sent to the mod
    pub events_in: VecDeque<In>,

    /// Events that have been sent to the host
    pub events_out: Vec<Out>,
}

struct WasmRuntime<In: Message, Out: Message> {
    instance: Instance,
    store: Store<State<In, Out>>,
}

/// Resource used for interacting with mods
///
/// It is currently very bare-bones, and will be expanded in the future.
///
/// Insert a new mod at any time with [`WasmResource::insert_wasm`].
#[derive(Resource)]
pub struct WasmResource<In: Message, Out: Message> {
    protocol_version: Version,
    runtimes: Vec<WasmRuntime<In, Out>>,
    engine: Engine,
}

impl<In: Message, Out: Message> WasmResource<In, Out> {
    /// Create a new WasmResource with a default engine
    pub fn new(protocol_version: Version) -> Self {
        let engine = Engine::default();
        WasmResource {
            protocol_version,
            runtimes: vec![],
            engine,
        }
    }

    /// Add a new mod
    pub fn insert_wasm(&mut self, wasm_bytes: impl AsRef<[u8]>) {
        // Create store and instance
        let module = Module::new(&self.engine, wasm_bytes).unwrap();
        let mut store = Store::new(
            &self.engine,
            State {
                startup_time: Instant::now(),
                app_ptr: 0,
                events_out: Vec::new(),
                events_in: VecDeque::new(),
            },
        );
        let mut linker = build_linker(&self.engine, self.protocol_version);
        linker.module(&mut store, "", &module).unwrap();
        let instance = linker.instantiate(&mut store, &module).unwrap();

        // Call wasm::build_app
        let build_app: TypedFunc<(), ()> =
            instance.get_typed_func(&mut store, "build_app").unwrap();
        match build_app.call(&mut store, ()) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to call build_app: {}", e);
                return;
            }
        }

        // Add the new runtime to the resource
        self.runtimes.push(WasmRuntime { instance, store });
    }
}

/// Add this plugin to your Bevy app to enable WASM-based modding
///
/// Give [`WasmPlugin::new`] a list of wasm files to load at startup.
/// Further mods can be added at any time with [`WasmResource::insert_wasm`].
pub struct WasmPlugin<In, Out>
where
    In: Message,
    Out: Message,
{
    wasm_bytes: Vec<Box<[u8]>>,
    protocol_version: Version,
    _in: std::marker::PhantomData<In>,
    _out: std::marker::PhantomData<Out>,
}

impl<In: Message, Out: Message> WasmPlugin<In, Out> {
    /// Create a WasmPlugin with a list of wasm files to load at startup
    pub fn new(protocol_version: Version) -> Self {
        info!(
            "Starting {}{}{}{} {}{}{}{} with protocol version {}.{}.{}",
            "B".bold().red(),
            "E".bold().yellow(),
            "V".bold().green(),
            "Y".bold().cyan(),
            "W".bold().blue(),
            "A".bold().magenta(),
            "S".bold().red(),
            "M".bold().yellow(),
            protocol_version.major,
            protocol_version.minor,
            protocol_version.patch,
        );
        WasmPlugin {
            wasm_bytes: Vec::new(),
            protocol_version: protocol_version.into(),
            _in: std::marker::PhantomData,
            _out: std::marker::PhantomData,
        }
    }

    /// Add a wasm file to the plugin
    pub fn with_mod(mut self, wasm: impl Into<Box<[u8]>>) -> Self {
        self.wasm_bytes.push(wasm.into());
        self
    }
}

impl<In: Message, Out: Message> Plugin for WasmPlugin<In, Out> {
    fn build(&self, app: &mut App) {
        let mut wasm_resource = WasmResource::<In, Out>::new(self.protocol_version.clone().into());

        for wasm_bytes in &self.wasm_bytes {
            wasm_resource.insert_wasm(wasm_bytes);
        }

        app.insert_resource(wasm_resource)
            .add_event::<In>()
            .add_event::<Out>()
            .add_system(update_system::<In, Out>)
            .add_system_to_stage(CoreStage::PostUpdate, event_listener::<In, Out>);
    }
}

fn update_system<In: Message, Out: Message>(
    mut wasm: ResMut<WasmResource<In, Out>>,
    mut events: EventWriter<Out>,
) {
    for runtime in wasm.runtimes.iter_mut() {
        let update: TypedFunc<i32, ()> = runtime
            .instance
            .get_typed_func(&mut runtime.store, "update")
            .unwrap();
        let app_ptr = runtime.store.data().app_ptr;
        update.call(&mut runtime.store, app_ptr).unwrap();
        let events_out = std::mem::take(&mut runtime.store.data_mut().events_out);
        for event in events_out {
            events.send(event);
        }
    }
}

fn event_listener<In: Message, Out: Message>(
    mut wasm: ResMut<WasmResource<In, Out>>,
    mut events: EventReader<In>,
) {
    for event in events.iter() {
        for runtime in wasm.runtimes.iter_mut() {
            runtime.store.data_mut().events_in.push_back(event.clone());
        }
    }
}

fn build_linker<In: Message, Out: Message>(
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
                info!("{} {}", "MOD".bold().green(), string);
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
                warn!("{} {}", "MOD".bold().yellow(), string);
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
                error!("{} {}", "MOD".bold().red(), string);
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

                let data = mem
                    .data_mut(&mut caller)
                    .get_mut(arena as u32 as usize..)
                    .and_then(|arr| arr.get_mut(..len as u32 as usize))
                    .unwrap();

                data[..serialized_event.len()].copy_from_slice(serialized_event.as_slice());
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

/// Convinience exports
pub mod prelude {
    pub use crate::{Message, WasmPlugin};
    pub use bevy_wasm_shared::prelude::*;
}
