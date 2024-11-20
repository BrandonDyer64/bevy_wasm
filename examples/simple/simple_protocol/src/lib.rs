use bevy::prelude::Event;
use bevy_wasm_shared::prelude::*;
use serde::{Deserialize, Serialize};

/// The version of the protocol. Automatically set from the `CARGO_PKG_VERSION` environment variable.
pub const PROTOCOL_VERSION: Version = version!();

/// A message to be sent Mod -> Game.
#[derive(Event, Clone, Serialize, Deserialize, Debug)]
pub enum ModMessage {
    Hello,
}

/// A message to be sent Game -> Mod.
#[derive(Event, Clone, Serialize, Deserialize, Debug)]
pub enum GameMessage {
    HiThere,
}
