use bevy::{prelude::*, utils::HashMap};

pub struct ResourceMutationBundle {
    pub serialized_values: HashMap<String, Box<[u8]>>,
    pub requester: Entity,
}
