use std::{collections::VecDeque, sync::Arc, time::Instant};

use anyhow::{Context, Result};
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
    pub fn new(engine: &WasmEngine, wasm_bytes: impl AsRef<[u8]>) -> Result<Self> {
        // Create store and instance
        let module = Module::new(&engine.engine(), wasm_bytes)?;
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
        let instance = build_linker(&engine.engine(), engine.protocol_version())
            .context("Failed to build a linker for bevy_wasm")?
            .module(&mut store, "", &module)?
            .instantiate(&mut store, &module)?;

        // Call `extern "C" fn build_app`
        instance
            .get_typed_func::<(), ()>(&mut store, "build_app")?
            .call(&mut store, ())
            .context("Failed to call build_app")?;

        Ok(Self { instance, store })
    }

    /// Send a serialized event to this mod
    pub fn send_serialized_event(&mut self, event: Arc<[u8]>) {
        self.store.data_mut().events_in.push_back(event);
    }

    /// Tick the internal mod state
    pub(crate) fn tick(&mut self, events_in: &[Arc<[u8]>]) -> Result<Vec<Box<[u8]>>> {
        for event in events_in.iter() {
            self.store.data_mut().events_in.push_back(event.clone());
        }

        let app_ptr = self.store.data().app_ptr;

        // Call `extern "C" fn update`
        self.instance
            .get_typed_func::<i32, ()>(&mut self.store, "update")?
            .call(&mut self.store, app_ptr)
            .context("Failed to call update")?;

        let serialized_events_out = std::mem::take(&mut self.store.data_mut().events_out);

        Ok(serialized_events_out)
    }

    /// Update the value of a shared resource as seen by the mod
    pub fn update_resource_value(&mut self, resource_name: &String, resource_bytes: Arc<[u8]>) {
        let state = self.store.data_mut();

        state
            .shared_resource_values
            .insert(resource_name.to_string(), resource_bytes);
    }
}
