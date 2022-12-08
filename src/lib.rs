use bevy_app::{App, Plugin};
use bevy_ecs::system::{Commands, ResMut, Resource};
use bevy_log::{info, trace};
use wasmtime::*;

struct State {
    app_ptr: i32,
}

#[derive(Resource)]
struct WasmResource {
    instance: Instance,
    store: Store<State>,
}

pub struct WasmPlugin(pub &'static [u8]);

impl Plugin for WasmPlugin {
    fn build(&self, app: &mut App) {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
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
        let module = Module::new(&engine, self.0).unwrap();
        let mut store = Store::new(&engine, State { app_ptr: 0 });
        linker.module(&mut store, "", &module).unwrap();
        let instance = linker.instantiate(&mut store, &module).unwrap();

        app.insert_resource(WasmResource { instance, store })
            .add_startup_system(startup_system)
            .add_system(update_system);
    }
}

fn startup_system(wasm: ResMut<WasmResource>) {
    let resource = wasm.into_inner();
    let build_app: TypedFunc<(), ()> = resource
        .instance
        .get_typed_func(&mut resource.store, "build_app")
        .unwrap();
    build_app.call(&mut resource.store, ()).unwrap();
}

fn update_system(wasm: ResMut<WasmResource>) {
    let resource = wasm.into_inner();
    let update: TypedFunc<i32, ()> = resource
        .instance
        .get_typed_func(&mut resource.store, "update")
        .unwrap();
    let app_ptr = resource.store.data().app_ptr;
    update.call(&mut resource.store, app_ptr).unwrap();
}
