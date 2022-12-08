use bevy_app::{App, Plugin};
use bevy_ecs::{
    prelude::EventWriter,
    system::{ResMut, Resource},
};
use bevy_log::{error, info, warn};
use serde::{de::DeserializeOwned, Serialize};
use wasmtime::*;

pub trait Message: Send + Sync + Serialize + DeserializeOwned + 'static {}

impl<T> Message for T where T: Send + Sync + Serialize + DeserializeOwned + 'static {}

struct State<M: Message> {
    app_ptr: i32,
    events_out: Vec<M>,
}

struct WasmRuntime<M: Message> {
    instance: Instance,
    store: Store<State<M>>,
}

#[derive(Resource)]
pub struct WasmResource<M: Message> {
    runtimes: Vec<WasmRuntime<M>>,
    linker: Linker<State<M>>,
    engine: Engine,
}

impl<M: Message> WasmResource<M> {
    pub fn new() -> Self {
        let engine = Engine::default();
        let mut linker: Linker<State<M>> = Linker::new(&engine);
        linker
            .func_wrap(
                "host",
                "console_info",
                |mut caller: Caller<'_, State<M>>, msg: i32, len: i32| {
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
                "host",
                "console_warn",
                |mut caller: Caller<'_, State<M>>, msg: i32, len: i32| {
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
                    warn!("{}", string);
                },
            )
            .unwrap();
        linker
            .func_wrap(
                "host",
                "console_error",
                |mut caller: Caller<'_, State<M>>, msg: i32, len: i32| {
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
                    error!("{}", string);
                },
            )
            .unwrap();
        linker
            .func_wrap(
                "host",
                "store_app",
                |mut caller: Caller<'_, State<M>>, app_ptr: i32| {
                    caller.data_mut().app_ptr = app_ptr;
                    info!("Storing app pointer: {:X}", app_ptr);
                },
            )
            .unwrap();
        linker
            .func_wrap(
                "host",
                "send_serialized_event",
                |mut caller: Caller<'_, State<M>>, msg: i32, len: i32| {
                    let mem = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => mem,
                        _ => panic!("failed to find host memory"),
                    };

                    let data = mem
                        .data(&caller)
                        .get(msg as u32 as usize..)
                        .and_then(|arr| arr.get(..len as u32 as usize))
                        .unwrap();
                    let event: M = match bincode::deserialize(data) {
                        Ok(event) => event,
                        Err(e) => {
                            error!("Failed to deserialize event: {}", e);
                            return;
                        }
                    };
                    caller.data_mut().events_out.push(event);
                },
            )
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
                |mut caller: Caller<'_, State<M>>, msg: i32, len: i32| {
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
        WasmResource {
            runtimes: vec![],
            linker,
            engine,
        }
    }

    pub fn insert_wasm(&mut self, wasm_bytes: impl AsRef<[u8]>) {
        // Create store and instance
        let module = Module::new(&self.engine, wasm_bytes).unwrap();
        let mut store = Store::new(
            &self.engine,
            State {
                app_ptr: 0,
                events_out: vec![],
            },
        );
        self.linker.module(&mut store, "", &module).unwrap();
        let instance = self.linker.instantiate(&mut store, &module).unwrap();

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

pub struct WasmPlugin<M: Message>(pub Vec<Box<[u8]>>, pub std::marker::PhantomData<M>);

impl<M: Message> WasmPlugin<M> {
    pub fn new(wasm_bytes: Vec<Box<[u8]>>) -> Self {
        WasmPlugin(wasm_bytes, std::marker::PhantomData)
    }
}

impl<M: Message> Plugin for WasmPlugin<M> {
    fn build(&self, app: &mut App) {
        let mut wasm_resource = WasmResource::<M>::new();

        for wasm_bytes in &self.0 {
            wasm_resource.insert_wasm(wasm_bytes);
        }

        app.insert_resource(wasm_resource)
            .add_system(update_system::<M>);
    }
}

fn update_system<M: Message>(mut wasm: ResMut<WasmResource<M>>, mut events: EventWriter<M>) {
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
