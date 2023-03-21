use std::{
    collections::VecDeque,
    future::Future,
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Context, Result};
use bevy::{
    prelude::{info, Component, Resource},
    utils::{HashMap, Instant},
};
use js_sys::{
    Function, Object, Reflect,
    WebAssembly::{self, Instance},
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use bevy_wasm_shared::version::Version;
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
        info!("ATTEMPTING TO CREATE INSTANCE");
        let memory = Arc::new(RwLock::new(None));
        let mod_state = Arc::new(RwLock::new(ModState {
            startup_time: Instant::now(),
            app_ptr: 0,
            events_in: VecDeque::new(),
            events_out: Vec::new(),
            shared_resource_values: HashMap::new(),
        }));
        let imports = build_linker(self.protocol_version, mod_state.clone(), memory.clone());
        console::log_1(&imports);
        let promise = WebAssembly::instantiate_buffer(wasm_bytes, &imports);
        let instance = Arc::new(RwLock::new(None));
        let then = Closure::new({
            let instance = instance.clone();
            move |value| {
                info!("CLOSURE RESOLVED");
                console::log_1(&value);
                let instance_value: WebAssembly::Instance =
                    Reflect::get(&value, &"instance".into())
                        .and_then(|x| x.dyn_into())
                        .unwrap();
                let exports = instance_value.exports();
                let memory_value: WebAssembly::Memory = Reflect::get(&exports, &"memory".into())
                    .and_then(|x| x.dyn_into())
                    .unwrap();
                info!("Memory has {} pages", memory_value.grow(0));
                let build_app: Function = Reflect::get(exports.as_ref(), &"build_app".into())
                    .and_then(|x| x.dyn_into())
                    .expect("build_app export wasn't a function");
                console::log_2(&instance_value, &memory_value);
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
            then,
            catch,
        })
    }
}

#[derive(Component)]
pub struct WasmInstance {
    instance: Arc<RwLock<Option<Instance>>>,
    mod_state: Arc<RwLock<ModState>>,
    then: Closure<dyn FnMut(JsValue)>,
    catch: Closure<dyn FnMut(JsValue)>,
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
        update
            .call1(&JsValue::undefined(), &JsValue::from_f64(app_ptr as f64))
            .unwrap();

        let serialized_events_out = std::mem::take(&mut self.mod_state.write().unwrap().events_out);

        Ok(serialized_events_out)
    }

    pub fn update_resource_value<T: SharedResource>(&mut self, bytes: Arc<[u8]>) {
        self.mod_state
            .write()
            .unwrap()
            .shared_resource_values
            .insert(T::TYPE_UUID, bytes);
    }
}
