use std::{collections::VecDeque, sync::Arc};

use anyhow::{Context, Result};
use bevy::{
    prelude::{Component, Resource},
    utils::{HashMap, Instant},
};
use bevy_wasm_shared::version::Version;
use uuid::Uuid;
use wasi_common::sync::WasiCtxBuilder;
use wasi_common::WasiCtx;
use wasmtime::*;

use crate::{mod_state::ModState, SharedResource};

use self::linker::build_linker;

mod linker;

#[derive(Resource)]
pub struct WasmRuntime {
    engine: Engine,
    protocol_version: Version,
}

pub struct WasiModState {
    mod_state: ModState,
    wasi_ctx: WasiCtx,
}

impl WasmRuntime {
    pub fn new(protocol_version: Version) -> Self {
        Self {
            engine: Engine::default(),
            protocol_version,
        }
    }

    pub fn create_instance(&self, wasm_bytes: &[u8]) -> Result<WasmInstance> {
        // Create store and instance
        let module = Module::new(&self.engine, wasm_bytes)?;
        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()?
            .build();

        let mod_state = ModState {
            startup_time: Instant::now(),
            app_ptr: 0,
            events_out: Vec::new(),
            events_in: VecDeque::new(),
            shared_resource_values: HashMap::new(),
        };

        let wasi_mod_state = WasiModState {
            mod_state,
            wasi_ctx,
        };

        let mut store = Store::new(&self.engine, wasi_mod_state);

        let instance = build_linker(&self.engine, self.protocol_version)
            .context("Failed to build a linker for bevy_wasm")?
            .module(&mut store, "", &module)?
            .instantiate(&mut store, &module)?;

        // Call `extern "C" fn build_app`
        instance
            .get_typed_func::<(), ()>(&mut store, "build_app")?
            .call(&mut store, ())
            .context("Failed to call build_app")?;

        Ok(WasmInstance { instance, store })
    }
}

#[derive(Component)]
pub struct WasmInstance {
    instance: Instance,
    store: Store<WasiModState>,
}

impl WasmInstance {
    /// Tick the internal mod state
    pub(crate) fn tick(&mut self, events_in: &[Arc<[u8]>]) -> Result<Vec<Box<[u8]>>> {
        for event in events_in.iter() {
            self.store.data_mut().mod_state.events_in.push_back(event.clone());
        }

        let app_ptr = self.store.data().mod_state.app_ptr;

        // Call `extern "C" fn update`
        self.instance
            .get_typed_func::<i32, ()>(&mut self.store, "update")?
            .call(&mut self.store, app_ptr)
            .context("Failed to call update")?;

        let serialized_events_out = std::mem::take(&mut self.store.data_mut().mod_state.events_out);

        Ok(serialized_events_out)
    }

    /// Update the value of a shared resource as seen by the mod
    pub fn update_resource_value<T: SharedResource>(&mut self, bytes: Arc<[u8]>) {
        let state = self.store.data_mut();

        state
            .mod_state
            .shared_resource_values
            .insert(Uuid::from_bytes(T::UUID), bytes);
    }
}
