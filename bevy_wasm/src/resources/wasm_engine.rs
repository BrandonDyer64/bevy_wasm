//! The `WasmEngine` struct

use bevy::prelude::*;
use bevy_wasm_shared::prelude::*;
use wasmtime::*;

/// Resource used for interacting with mods
///
/// It is currently very bare-bones, and will be expanded in the future.
///
/// Insert a new mod at any time with [`WasmEngine::insert_wasm`].
#[derive(Resource)]
pub struct WasmEngine {
    protocol_version: Version,
    engine: Engine,
}

impl WasmEngine {
    /// Create a new WasmEngine with a default engine
    pub fn new(protocol_version: Version) -> Self {
        let engine = Engine::default();
        WasmEngine {
            protocol_version,
            engine,
        }
    }

    /// Get the protocol [`Version`] of the game
    pub fn protocol_version(&self) -> Version {
        self.protocol_version
    }

    /// Get the internal Wasmtime [`Engine`]
    pub fn engine(&self) -> Engine {
        self.engine.clone()
    }
}
