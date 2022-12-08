use serde::Serialize;

pub fn send_event<T: Serialize>(event: &T) {
    let encoded: Vec<u8> = bincode::serialize(&event).unwrap();
    unsafe {
        crate::ffi::send_serialized_event(encoded.as_ptr(), encoded.len());
    }
}
