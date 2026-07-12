
use std::any::type_name;
use xxhash_rust::xxh3::xxh3_128;


/// Cube Identification
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ResourceId(pub u64, pub u64);


/// 128 = (64, 64) usage
pub fn resource_id<T: ?Sized>() -> ResourceId {
    let hash = xxh3_128(type_name::<T>().as_bytes());

    ResourceId(
        (hash >> 64) as u64,
        hash as u64,
    )
}
