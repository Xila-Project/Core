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
