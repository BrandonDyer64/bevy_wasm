//! Event functions. [`send_event`] and [`get_next_event`]

use serde::{de::DeserializeOwned, Serialize};

use crate::error;

/// Send an event to the host.
pub fn send_event<T: Serialize>(event: &T) {
    let encoded: Vec<u8> = bincode::serialize(&event).unwrap();
    unsafe {
        crate::ffi::send_serialized_event(encoded.as_ptr(), encoded.len());
    }
}

/// Get the next event from the host.
pub fn get_next_event<T: DeserializeOwned>() -> Option<T> {
    unsafe {
        let mut event_arena: Vec<u8> = vec![0; 1024];
        let len = crate::ffi::get_next_event(event_arena.as_mut_ptr(), event_arena.len());
        if len == 0 {
            return None;
        }
        let event = match bincode::deserialize(&event_arena[..len]) {
            Ok(event) => event,
            Err(err) => {
                error!("Failed to deserialize event: {}", err);
                return None;
            }
        };
        Some(event)
    }
}
