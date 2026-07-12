use std::sync::Arc;
use tracing::{error, info, warn};
use bevy::prelude::*;
use bevy_ecs::message::{MessageReader, MessageWriter};

use crate::{runtime::WasmInstance, Message};

pub fn tick_mods<In: Message + bevy::prelude::Message, Out: Message + bevy::prelude::Message>(
    mut events_in: MessageReader<In>,
    mut events_out: MessageWriter<Out>,
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
/*
        for serialized_event_out in serialized_events_out {
            match bincode::deserialize(&serialized_event_out) {
                Ok(event_out) => events_out.write(event_out),
                Err(err) => error!("Error while deserializing event: {}", err),
            }
        }
*/
        for serialized_event_out in serialized_events_out {
            /*
            match bincode::deserialize(&serialized_event_out) {
                Ok(event_out) => events_out.send(event_out),
                // Err(err) => error!("Error while deserializing event: {}", err),
                //     Err(err) => {
                //
                //    error!("Error while deserializing event: {}", err);
                Err(e/rr) => error!("Error while deserializing event: {}", err),
            }
            */
            if let Err(err) = bincode::deserialize::<Out>(&serialized_event_out) {
                error!("Error while deserializing event: {}", err);
            } else if let Ok(event_out) = bincode::deserialize(&serialized_event_out) {
                // events_out.send(event_out);
                events_out.write(event_out);
            }
        }
    }
}
