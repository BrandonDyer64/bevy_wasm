//! This crate provides the shared code between the bevy_wasm and bevy_wasm_sys.
//!
//! Use this for your protocol crate.

#![deny(missing_docs)]

pub mod version;

/// Convenience re-exports
pub mod prelude {
    pub use crate::version;
    pub use crate::version::Version;
}
