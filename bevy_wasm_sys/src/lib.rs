/*!
Build mods for Bevy with Bevy.

This is the sys crate, intended to be used inside of mods.

To make a game that can use WebAssembly mods, see the sister crate [bevy_wasm](https://docs.rs/bevy_wasm) crate.
*/

#![deny(missing_docs)]

pub mod ecs;
pub mod ffi;
pub mod ffi_plugin;
pub mod time;

/// Convenience re-exports
pub mod prelude {
    pub use bevy::app::prelude::*;
    pub use bevy::ecs::prelude::*;
    pub use bevy::math::prelude::*;
    pub use bevy::reflect::prelude::*;
    pub use bevy::transform::prelude::*;
    pub use bevy_wasm_shared::prelude::*;
    pub use bevy_wasm_sys_core::{error, info, warn};

    pub use crate::ecs::prelude::*;
    pub use crate::ffi_plugin::FFIPlugin;
    pub use crate::time::Time;
}
