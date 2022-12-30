/*!
Build mods for Bevy with Bevy.

This is the sys crate, intended to be used inside of mods.

To make a game that can use WebAssembly mods, see the sister crate [bevy_wasm](https://docs.rs/bevy_wasm) crate.
*/

#![deny(missing_docs)]

pub mod events;
pub mod ffi;
pub mod macros;

#[cfg(feature = "bevy")]
pub mod ecs;

#[cfg(feature = "bevy")]
pub mod ffi_plugin;

#[cfg(feature = "bevy")]
pub mod time;

/// Convenience re-exports
pub mod prelude {
    pub use crate::macros::*;
    pub use crate::{error, info, warn};
    pub use bevy_wasm_shared::prelude::*;

    #[cfg(feature = "bevy")]
    pub use {
        crate::ecs::prelude::*, crate::ffi_plugin::FFIPlugin, crate::time::Time,
        bevy_app::prelude::*, bevy_derive::*, bevy_ecs::prelude::*, bevy_math::prelude::*,
        bevy_reflect::prelude::*, bevy_transform::prelude::*,
    };
}
