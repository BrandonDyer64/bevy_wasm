use bevy_app::{App, Plugin};
use bevy_ecs::system::{ResMut, Resource};
use bevy_log::info;
use wasmtime::*;

struct State {
    app_ptr: i32,
}

struct WasmRuntime {
    instance: Instance,
    store: Store<State>,
}

#[derive(Resource)]
pub struct WasmResource {
    runtimes: Vec<WasmRuntime>,
    linker: Linker<State>,
    engine: Engine,
}

impl WasmResource {
    pub fn new() -> Self {
        let engine = Engine::default();
        let mut linker: Linker<State> = Linker::new(&engine);
        linker
            .func_wrap(
                "host",
                "console_info",
                |mut caller: Caller<'_, State>, msg: i32, len: i32| {
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
                "store_app",
                |mut caller: Caller<'_, State>, app_ptr: i32| {
                    caller.data_mut().app_ptr = app_ptr;
                    info!("Storing app pointer: {:X}", app_ptr);
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
                |mut caller: Caller<'_, State>, msg: i32, len: i32| {
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
        let mut store = Store::new(&self.engine, State { app_ptr: 0 });
        self.linker.module(&mut store, "", &module).unwrap();
        let instance = self.linker.instantiate(&mut store, &module).unwrap();

        // Call wasm::build_app
        let build_app: TypedFunc<(), ()> =
            instance.get_typed_func(&mut store, "build_app").unwrap();
        build_app.call(&mut store, ()).unwrap();

        // Add the new runtime to the resource
        self.runtimes.push(WasmRuntime { instance, store });
    }
}

pub struct WasmPlugin(pub Vec<Box<[u8]>>);

impl Plugin for WasmPlugin {
    fn build(&self, app: &mut App) {
        let mut wasm_resource = WasmResource::new();

        for wasm_bytes in &self.0 {
            wasm_resource.insert_wasm(wasm_bytes);
        }

        app.insert_resource(wasm_resource).add_system(update_system);
    }
}

fn update_system(mut wasm: ResMut<WasmResource>) {
    for runtime in wasm.runtimes.iter_mut() {
        let update: TypedFunc<i32, ()> = runtime
            .instance
            .get_typed_func(&mut runtime.store, "update")
            .unwrap();
        let app_ptr = runtime.store.data().app_ptr;
        update.call(&mut runtime.store, app_ptr).unwrap();
    }
}
