use serde::{Deserialize, Serialize};

/// Messages passed `Host -> Mod`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostMessage {
    /// A cube was spawned. This is its entity id.
    SpawnedCube {
        /// mod-specific state, specified on [`ModMessage::SpawnCube`]
        mod_state: u64,
        entity_id: u32,
    },
}

/// Messages passed `Mod -> Host`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModMessage {
    /// Spawn a cube
    SpawnCube {
        /// This is a mod-specific state that will be passed back to the mod when the cube is spawned
        mod_state: u64,
        color: (f32, f32, f32),
    },
    /// Move a cube given an entity id
    MoveCube {
        entity_id: u32,
        x: f32,
        y: f32,
        z: f32,
    },
}
