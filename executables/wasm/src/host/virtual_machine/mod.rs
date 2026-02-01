mod custom_data;
mod environment;
mod error;
mod instance;
mod manager;
mod module;
mod registrable;
mod runtime;

// Re-export key types from WAMR
pub use wamr_rust_sdk::value::WasmValue;

// Re-export all public types from modules
pub use custom_data::*;
pub use environment::*;
pub use error::*;
pub use instance::*;
pub use manager::*;
pub use module::*;
pub use registrable::*;
pub use runtime::*;

/// Type alias for WASM pointer addresses in the 32-bit WASM address space
pub type WasmPointer = u32;

/// Type alias for WASM size values (32-bit)
pub type WasmUsize = u32;
