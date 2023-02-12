use bevy::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::events::{ResourceMutation, ResourceMutationBundle};

pub fn send_mutation_requests<T: Resource + DeserializeOwned + Serialize>(
    mut bundle_events: EventReader<ResourceMutationBundle>,
    mut event_writer: EventWriter<ResourceMutation<T>>,
) {
    for bundle in bundle_events.iter() {
        if let Some(serialized_value) = bundle.serialized_values.get(std::any::type_name::<T>()) {
            let value = match bincode::deserialize(serialized_value) {
                Ok(value) => value,
                Err(err) => {
                    error!("Error while deserializing resource mutation: {}", err);
                    continue;
                }
            };
            event_writer.send(ResourceMutation {
                proposed_value: value,
                requester: bundle.requester,
            })
        }
    }
}
