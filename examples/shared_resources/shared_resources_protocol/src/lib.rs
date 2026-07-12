
use bevy_reflect::Reflect;
use bevy_wasm_shared::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_ecs::{
    message::Message,
    prelude::Resource,
};
/// The version of the protocol. Automatically set from the `CARGO_PKG_XXX` environment variables.
pub const PROTOCOL_VERSION: Version = version!();

/// A resource that we want to share between the host and the mod
/// Must implement 'Reflect', `Resource` and `Serialize`/`Deserialize`
#[derive(Debug, Default, Clone, Resource, Serialize, Deserialize, Reflect)]
pub struct MyCoolResource {
    pub value: u32,
    pub string: String,
}

/// Messages passed `Host -> Mod`
#[derive(Debug, Message, Clone, Serialize, Deserialize)]
pub enum HostMessage {
    // We don't care about this right now
}

/// Messages passed `Mod -> Host`
#[derive(Debug, Message, Clone, Serialize, Deserialize)]
pub enum ModMessage {
    // We don't care about this right now
}
