//! This crate provides the shared code between the bevy_wasm and bevy_wasm_sys.
//!
//! Use this for your protocol crate.

#![deny(missing_docs)]

pub mod version;
/// Cube Identification
pub mod resource_id;
/// Convenience re-exports
pub mod prelude {
    pub use crate::version;
    pub use crate::version::Version;
}
