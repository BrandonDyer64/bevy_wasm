use std::ops::Deref;

use bevy_ecs::prelude::*;
use bevy_log::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use wasmtime::*;

use crate::{resource::WasmResource, Message};

pub(crate) fn update_mods<In: Message, Out: Message>(
    mut wasm: ResMut<WasmResource<In, Out>>,
    mut events: EventWriter<Out>,
) {
    for runtime in wasm.runtimes.iter_mut() {
        let update: TypedFunc<i32, ()> = runtime
            .instance
            .get_typed_func(&mut runtime.store, "update")
            .unwrap();
        let app_ptr = runtime.store.data().app_ptr;
        match update.call(&mut runtime.store, app_ptr) {
            Ok(_) => {}
            Err(e) => error!("Error calling mod update:\n{}", e),
        }
        let events_out = std::mem::take(&mut runtime.store.data_mut().events_out);
        for event in events_out {
            events.send(event);
        }
    }
}

pub(crate) fn update_shared_resource<
    T: Resource + DeserializeOwned + Serialize,
    In: Message,
    Out: Message,
>(
    res: Res<T>,
    mut wasm: ResMut<WasmResource<In, Out>>,
) {
    if res.is_changed() {
        let v: &T = res.deref();
        let resource_bytes: Vec<u8> = bincode::serialize(v).unwrap();
        wasm.update_resource::<T>(resource_bytes);
    }
}

pub(crate) fn event_listener<In: Message, Out: Message>(
    mut wasm: ResMut<WasmResource<In, Out>>,
    mut events: EventReader<In>,
) {
    for event in events.iter() {
        for runtime in wasm.runtimes.iter_mut() {
            runtime.store.data_mut().events_in.push_back(event.clone());
        }
    }
}
