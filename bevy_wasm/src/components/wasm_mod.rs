use std::{collections::VecDeque, sync::Arc, time::Instant};

use bevy::{prelude::*, utils::HashMap};
use wasmtime::*;

use crate::{linker::build_linker, mod_state::ModState, prelude::WasmEngine};

/// A WebAssembly mod
#[derive(Component)]
pub struct WasmMod {
    instance: Instance,
    store: Store<ModState>,
}

impl WasmMod {
    /// Create a new Webassembly mod component. Be sure to add this to an entity.
    pub fn new(engine: &WasmEngine, wasm_bytes: impl AsRef<[u8]>) -> Result<Self, ()> {
        // Create store and instance
        let module = Module::new(&engine.engine(), wasm_bytes).unwrap();
        let mut store = Store::new(
            &engine.engine(),
            ModState {
                startup_time: Instant::now(),
                app_ptr: 0,
                events_out: Vec::new(),
                events_in: VecDeque::new(),
                shared_resource_values: HashMap::new(),
            },
        );
        let mut linker: Linker<ModState> =
            build_linker(&engine.engine(), engine.protocol_version());
        linker.module(&mut store, "", &module).unwrap();
        let instance = linker.instantiate(&mut store, &module).unwrap();

        // Call wasm::build_app
        let build_app: TypedFunc<(), ()> =
            instance.get_typed_func(&mut store, "build_app").unwrap();
        match build_app.call(&mut store, ()) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to call build_app: {}", e);
                return Err(());
            }
        }

        Ok(Self { instance, store })
    }

    /// Send a serialized event to this mod
    pub fn send_serialized_event(&mut self, event: Arc<[u8]>) {
        self.store.data_mut().events_in.push_back(event);
    }

    /// Tick the internal mod state
    pub(crate) fn tick(&mut self, events_in: &[Arc<[u8]>]) -> Vec<Arc<[u8]>> {
        for event in events_in.iter() {
            self.store.data_mut().events_in.push_back(event.clone());
        }

        let update_fn: TypedFunc<i32, ()> = self
            .instance
            .get_typed_func(&mut self.store, "update")
            .unwrap();
        let app_ptr = self.store.data().app_ptr;
        match update_fn.call(&mut self.store, app_ptr) {
            Ok(_) => {}
            Err(e) => error!("Error calling mod update:\n{}", e),
        }

        let events_out = std::mem::take(&mut self.store.data_mut().events_out);
        events_out
    }

    /// Update the value of a shared resource as seen by the mod
    pub fn update_resource_value(&mut self, resource_name: &String, resource_bytes: Arc<[u8]>) {
        let state = self.store.data_mut();

        state
            .shared_resource_values
            .insert(resource_name.to_string(), resource_bytes);
    }
}
