/*!
Build mods for bevy with bevy.

This is the sys crate, intended to be used inside of mods.
*/

#![deny(missing_docs)]

pub mod events;
pub mod ffi;
pub mod macros;

#[cfg(feature = "bevy")]
pub mod ffi_plugin;

/// Convenience re-exports
pub mod prelude {
    pub use crate::macros::*;

    #[cfg(feature = "bevy")]
    pub use {crate::ffi_plugin::FFIPlugin, bevy_app::prelude::*, bevy_ecs::prelude::*};
}
