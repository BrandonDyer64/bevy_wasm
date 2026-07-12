use std::{collections::VecDeque, sync::Arc};
use bevy_wasm_shared::resource_id::{ResourceId, resource_id};
// use bevy::utils::{HashMap, Instant, Uuid};
use std::collections::HashMap;
use std::time::Instant;
/// Internal mod state
pub struct ModState {
    /// Time when the mod was loaded
    pub startup_time: Instant,

    /// Pointer given to us in `store_app`
    pub app_ptr: i32,

    /// Events that have been sent to the mod
    pub events_in: VecDeque<Arc<[u8]>>,

    /// Events that have been sent to the host
    pub events_out: Vec<Box<[u8]>>,

    /// Resources that have changed since the last update
    pub shared_resource_values: HashMap<ResourceId, Arc<[u8]>>,
    //pub shared_resource_values: HashMap<Uuid, Arc<[u8]>>,
}
