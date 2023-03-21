#[cfg(target_arch = "wasm32")]
pub use web::{WasmInstance, WasmRuntime};

#[cfg(not(target_arch = "wasm32"))]
pub use native::{WasmInstance, WasmRuntime};

#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
