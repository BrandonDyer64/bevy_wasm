//! The `WasmResource` struct

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Instant,
};

use bevy_ecs::prelude::*;
use bevy_log::prelude::*;
use bevy_wasm_shared::prelude::*;
use serde::Serialize;
use wasmtime::*;

use crate::{linker::build_linker, Message};

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

    /// Resources that have changed since the last update
    pub shared_resource_values: HashMap<String, Arc<Vec<u8>>>,
}

pub(crate) struct WasmRuntime<In: Message, Out: Message> {
    pub instance: Instance,
    pub store: Store<State<In, Out>>,
}

/// Resource used for interacting with mods
///
/// It is currently very bare-bones, and will be expanded in the future.
///
/// Insert a new mod at any time with [`WasmResource::insert_wasm`].
#[derive(Resource)]
pub struct WasmResource<In: Message, Out: Message> {
    pub(crate) protocol_version: Version,
    pub(crate) runtimes: Vec<WasmRuntime<In, Out>>,
    pub(crate) engine: Engine,
    pub(crate) shared_resources: HashMap<String, Arc<Vec<u8>>>,
}

impl<In: Message, Out: Message> WasmResource<In, Out> {
    /// Create a new WasmResource with a default engine
    pub fn new(protocol_version: Version) -> Self {
        let engine = Engine::default();
        WasmResource {
            protocol_version,
            runtimes: vec![],
            engine,
            shared_resources: HashMap::new(),
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
                shared_resource_values: HashMap::new(),
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

    /// Update the value of a shared resource
    pub fn update_resource<R: Resource + Serialize>(&mut self, resource_bytes: Vec<u8>) {
        info!("Serialized bytes: {:?}", resource_bytes);
        let resource_name = std::any::type_name::<R>();

        let resource_rc = Arc::new(resource_bytes);

        self.shared_resources
            .insert(resource_name.to_string(), resource_rc.clone());

        for runtime in self.runtimes.iter_mut() {
            let state = runtime.store.data_mut();

            state
                .shared_resource_values
                .insert(resource_name.to_string(), resource_rc.clone());
        }
    }
}
