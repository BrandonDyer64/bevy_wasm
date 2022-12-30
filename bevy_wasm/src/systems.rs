use bevy_ecs::prelude::*;
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
        update.call(&mut runtime.store, app_ptr).unwrap();
        let events_out = std::mem::take(&mut runtime.store.data_mut().events_out);
        for event in events_out {
            events.send(event);
        }
    }
}

pub(crate) fn update_shared_system<
    T: Resource + DeserializeOwned + Serialize,
    In: Message,
    Out: Message,
>(
    res: Res<T>,
    mut wasm: ResMut<WasmResource<In, Out>>,
) {
    if res.is_changed() {
        wasm.shared_resources.insert(
            std::any::type_name::<T>().to_string(),
            bincode::serialize(&*res).unwrap(),
        );
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
