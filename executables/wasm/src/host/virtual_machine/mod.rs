mod custom_data;
mod environment;
mod error;
mod instance;
mod module;
mod registrable;
mod runtime;
mod translation;

// Re-export all public types from modules
pub use custom_data::*;
pub use environment::*;
pub use error::*;
pub use instance::*;
pub use module::*;
pub use registrable::*;
pub use runtime::*;
pub use translation::*;

#[cfg(feature = "wasm32")]
/// Type alias for WASM pointer addresses in the 32-bit WASM address space
pub type WasmPointer = u32;
#[cfg(feature = "wasm64")]
pub type WasmPointer = u64;

#[cfg(feature = "wasm32")]
/// Type alias for WASM size values (32-bit)
pub type WasmUsize = u32;

#[cfg(feature = "wasm64")]
pub type WasmUsize = u64;
