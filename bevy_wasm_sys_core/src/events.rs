//! Event functions. [`send_event`] and [`get_next_event`]

use serde::{de::DeserializeOwned, Serialize};

use crate::error;

/// Send an event to the host.
pub fn send_event<T: Serialize>(event: &T) {
    let encoded: Vec<u8> = match bincode::serialize(&event) {
        Ok(encoded) => encoded,
        Err(err) => {
            error!("Failed to serialize event: {}", err);
            return;
        }
    };

    unsafe {
        crate::ffi::send_serialized_event(encoded.as_ptr(), encoded.len());
    }

    std::mem::drop(encoded);
}

/// Get the next event from the host.
pub fn get_next_event<T: DeserializeOwned>() -> Option<T> {
    let mut buffer = [0; 1024];

    let len = unsafe { crate::ffi::get_next_event(buffer.as_mut_ptr(), buffer.len()) };

    if len == 0 {
        return None;
    }

    if len > buffer.len() {
        error!("Serialized event is larger than buffer");
        return None;
    }

    match bincode::deserialize(&buffer[..len]) {
        Ok(event) => Some(event),
        Err(err) => {
            error!("Failed to deserialize event from host: {}", err);
            None
        }
    }
}
