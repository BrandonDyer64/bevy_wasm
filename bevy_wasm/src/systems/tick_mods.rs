use std::sync::Arc;

use bevy::prelude::*;

use crate::{prelude::WasmMod, Message};

pub fn tick_mods<In: Message, Out: Message>(
    mut events_in: EventReader<In>,
    mut events_out: EventWriter<Out>,
    mut wasm_mods: Query<&mut WasmMod>,
) {
    let serialized_events_in: Vec<Arc<[u8]>> = events_in
        .iter()
        .flat_map(|event| bincode::serialize(event))
        .map(|bytes| bytes.into())
        .collect();

    for mut wasm_mod in wasm_mods.iter_mut() {
        let serialized_events_out = wasm_mod.tick(serialized_events_in.as_slice());
        for serialized_event_out in serialized_events_out {
            let event_out: Out = bincode::deserialize(&serialized_event_out).unwrap();
            events_out.send(event_out);
        }
    }
}
