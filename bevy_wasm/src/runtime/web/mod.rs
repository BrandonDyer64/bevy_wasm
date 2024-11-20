use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use anyhow::Result;
use bevy::{
    prelude::{Component, Resource},
    utils::{HashMap, Instant},
};
use js_sys::{
    Function, Reflect,
    WebAssembly::{self, Instance},
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

use bevy_wasm_shared::version::Version;
use uuid::Uuid;
use web_sys::console;

use crate::{mod_state::ModState, SharedResource};

use self::linker::build_linker;

mod linker;

#[derive(Resource)]
pub struct WasmRuntime {
    protocol_version: Version,
}

impl WasmRuntime {
    pub fn new(protocol_version: Version) -> Self {
        Self { protocol_version }
    }

    pub fn create_instance(&self, wasm_bytes: &[u8]) -> Result<WasmInstance> {
        let memory = Arc::new(RwLock::new(None));
        let mod_state = Arc::new(RwLock::new(ModState {
            startup_time: Instant::now(),
            app_ptr: 0,
            events_in: VecDeque::new(),
            events_out: Vec::new(),
            shared_resource_values: HashMap::new(),
        }));
        let imports = build_linker(self.protocol_version, mod_state.clone(), memory.clone());
        let promise = WebAssembly::instantiate_buffer(wasm_bytes, &imports);
        let instance = Arc::new(RwLock::new(None));
        let then = Closure::new({
            let instance = instance.clone();
            move |value| {
                let instance_value: WebAssembly::Instance =
                    Reflect::get(&value, &"instance".into())
                        .and_then(|x| x.dyn_into())
                        .unwrap();
                let exports = instance_value.exports();
                let memory_value: WebAssembly::Memory = Reflect::get(&exports, &"memory".into())
                    .and_then(|x| x.dyn_into())
                    .unwrap();
                let build_app: Function = Reflect::get(exports.as_ref(), &"build_app".into())
                    .and_then(|x| x.dyn_into())
                    .expect("build_app export wasn't a function");
                *instance.write().unwrap() = Some(instance_value);
                *memory.write().unwrap() = Some(memory_value);
                build_app.call0(&JsValue::undefined()).unwrap();
            }
        });
        let catch = Closure::new({
            move |value| {
                console::warn_1(&value);
            }
        });
        _ = promise.then(&then).catch(&catch);
        Ok(WasmInstance {
            instance,
            mod_state,
            _then: then,
            _catch: catch,
        })
    }
}

#[derive(Component)]
pub struct WasmInstance {
    instance: Arc<RwLock<Option<Instance>>>,
    mod_state: Arc<RwLock<ModState>>,
    _then: Closure<dyn FnMut(JsValue)>,
    _catch: Closure<dyn FnMut(JsValue)>,
}

unsafe impl Send for WasmInstance {}
unsafe impl Sync for WasmInstance {}

impl WasmInstance {
    pub fn tick(&mut self, events_in: &[Arc<[u8]>]) -> Result<Vec<Box<[u8]>>> {
        let Some(instance) = self.instance.read().unwrap().clone() else { return Ok(Vec::new()) };
        for event in events_in.iter() {
            self.mod_state
                .write()
                .unwrap()
                .events_in
                .push_back(event.clone());
        }

        let app_ptr = self.mod_state.read().unwrap().app_ptr;

        let exports = instance.exports();

        let update: Function = Reflect::get(exports.as_ref(), &"update".into())
            .and_then(|x| x.dyn_into())
            .expect("build_app export wasn't a function");
        match update.call1(&JsValue::undefined(), &JsValue::from_f64(app_ptr as f64)) {
            Ok(_) => {}
            Err(e) => console::error_1(&e),
        }

        let serialized_events_out = std::mem::take(&mut self.mod_state.write().unwrap().events_out);

        Ok(serialized_events_out)
    }

    pub fn update_resource_value<T: SharedResource>(&mut self, bytes: Arc<[u8]>) {
        self.mod_state
            .write()
            .unwrap()
            .shared_resource_values
            .insert(Uuid::from_bytes(T::UUID), bytes);
    }
}
