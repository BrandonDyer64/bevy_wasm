/*!
Build mods for Bevy with Bevy.

This is the sys crate, intended to be used inside of mods.

To make a game that can use WebAssembly mods, see the sister crate [bevy_wasm](https://docs.rs/bevy_wasm) crate.
*/

#![deny(missing_docs)]

pub mod events;
pub mod ffi;
pub mod macros;

/// Convenience re-exports
pub mod prelude {
    pub use bevy_wasm_shared::prelude::*;

    pub use crate::{error, info, warn};
}
