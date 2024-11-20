//! Mod Bevy games with WebAssembly
//!
//! See [examples/cubes](https://github.com/BrandonDyer64/bevy_wasm/tree/main/examples/cubes)
//! for a comprehensive example of how to use this.
//!
//! For building mods, see the sister crate [bevy_wasm_sys](https://docs.rs/bevy_wasm_sys).

#![deny(missing_docs)]

use bevy::prelude::{Event, Resource};
use serde::{de::DeserializeOwned, Serialize};
use type_uuid::TypeUuid;

pub mod components;
mod mod_state;
pub mod plugin;
mod runtime;
mod systems;
mod wasm_asset;

/// Any data type that can be used as a Host <-> Mod message
///
/// Must be [`Clone`], [`Send`], and [`Sync`], and must be (de)serializable with serde.
///
/// `bevy_wasm` uses `bincode` for serialization, so it's relatively fast.
pub trait Message: Event + Send + Sync + Serialize + DeserializeOwned + Clone + 'static {}

impl<T> Message for T where T: Event + Send + Sync + Serialize + DeserializeOwned + Clone + 'static {}

/// Any data type that can be used as a shared resource from Host to Mod
///
/// Must be [`Clone`], [`Send`], [`Sync`], and [`TypeUuid`], and must be (de)serializable with serde.
pub trait SharedResource: Resource + Serialize + DeserializeOwned + TypeUuid {}

impl<T> SharedResource for T where T: Resource + Serialize + DeserializeOwned + TypeUuid {}

/// Convinience exports
pub mod prelude {
    pub use crate::{components::*, plugin::WasmPlugin, Message};
    pub use bevy_wasm_shared::prelude::*;
}
