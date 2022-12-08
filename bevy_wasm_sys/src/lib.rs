pub mod events;
pub mod ffi;
pub mod macros;

#[cfg(feature = "bevy")]
pub mod ffi_plugin;

pub mod prelude {
    pub use crate::macros::*;

    #[cfg(feature = "bevy")]
    pub use {crate::ffi_plugin::FFIPlugin, bevy_app::prelude::*, bevy_ecs::prelude::*};
}
