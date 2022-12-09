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
    let mut buffer: Vec<u8> = vec![0; 1024];
    let event = unsafe {
        let len = crate::ffi::get_next_event(buffer.as_mut_ptr(), buffer.len());
        if len == 0 {
            return None;
        }
        if len > buffer.len() {
            error!("Serialized event is larger than buffer");
            return None;
        }
        let event = match bincode::deserialize(&buffer[..len]) {
            Ok(event) => event,
            Err(err) => {
                error!("Failed to deserialize event from host: {}", err);
                return None;
            }
        };
        event
    };
    std::mem::drop(buffer); // Ensure the `unsafe` shenanigans don't stop buffer from being dropped
    Some(event)
}
