pub mod ffi;
pub mod ffi_plugin;
pub mod macros;

pub mod prelude {
    pub use crate::{ffi_plugin::FFIPlugin, macros::*};
    pub use bevy_app::prelude::*;
    pub use bevy_ecs::prelude::*;
}
