use std::sync::Arc;

use bevy::prelude::*;

use crate::{runtime::WasmInstance, Message};

pub fn tick_mods<In: Message, Out: Message>(
    mut events_in: EventReader<In>,
    mut events_out: EventWriter<Out>,
    mut wasm_mods: Query<&mut WasmInstance>,
) {
    let serialized_events_in: Vec<Arc<[u8]>> = events_in
        .read()
        .flat_map(|event| bincode::serialize(event))
        .map(|bytes| bytes.into())
        .collect();

    for mut wasm_mod in wasm_mods.iter_mut() {
        let serialized_events_out = match wasm_mod.tick(serialized_events_in.as_slice()) {
            Ok(events) => events,
            Err(err) => {
                error!("Error while ticking mod: {}", err);
                continue;
            }
        };

        for serialized_event_out in serialized_events_out {
            match bincode::deserialize(&serialized_event_out) {
                Ok(event_out) => {
                    _ = events_out.send(event_out)
                },
                Err(err) => error!("Error while deserializing event: {}", err),
            }
        }
    }
}
