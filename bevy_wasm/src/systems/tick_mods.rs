use std::sync::Arc;

use bevy::prelude::*;

use crate::{
    events::ResourceMutationBundle,
    prelude::{ModTickResponse, WasmMod},
    Message,
};

pub fn tick_mods<In: Message, Out: Message>(
    mut events_in: EventReader<In>,
    mut events_out: EventWriter<Out>,
    mut wasm_mods: Query<(Entity, &mut WasmMod)>,
    mut mutation_bundle_events: EventWriter<ResourceMutationBundle>,
) {
    let serialized_events_in: Vec<Arc<[u8]>> = events_in
        .iter()
        .flat_map(|event| bincode::serialize(event))
        .map(|bytes| bytes.into())
        .collect();

    for (entity, mut wasm_mod) in wasm_mods.iter_mut() {
        let tick_response = match wasm_mod.tick(serialized_events_in.as_slice()) {
            Ok(events) => events,
            Err(err) => {
                error!("Error while ticking mod: {}", err);
                continue;
            }
        };

        let ModTickResponse {
            serialized_events_out,
            resource_mutation_requests,
        } = tick_response;

        for serialized_event_out in serialized_events_out {
            match bincode::deserialize(&serialized_event_out) {
                Ok(event_out) => events_out.send(event_out),
                Err(err) => error!("Error while deserializing event: {}", err),
            }
        }

        if !resource_mutation_requests.is_empty() {
            mutation_bundle_events.send(ResourceMutationBundle {
                serialized_values: resource_mutation_requests,
                requester: entity,
            })
        }
    }
}
