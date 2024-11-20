use bevy::ecs::prelude::*;
use bevy_wasm_shared::prelude::*;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

/// The version of the protocol. Automatically set from the `CARGO_PKG_XXX` environment variables.
pub const PROTOCOL_VERSION: Version = version!();

/// A resource that we want to share between the host and the mod
// Must implement `Resource` and `Serialize`/`Deserialize`
#[derive(Debug, Default, Clone, Resource, Serialize, Deserialize, TypeUuid)]
#[uuid = "e6f89ac2-8299-4c0a-8754-c404f14dae44"]
pub struct MyCoolResource {
    pub value: u32,
    pub string: String,
}

/// Messages passed `Host -> Mod`
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub enum HostMessage {
    // We don't care about this right now
}

/// Messages passed `Mod -> Host`
#[derive(Event, Debug, Clone, Serialize, Deserialize)]
pub enum ModMessage {
    // We don't care about this right now
}
