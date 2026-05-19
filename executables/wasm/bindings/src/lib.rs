#![no_std]

extern crate alloc;

mod context;
mod file_system;
mod fundamentals;
mod mutex;
mod task;

pub use context::*;
pub use context::*;
pub use file_system::*;
pub use fundamentals::*;
pub use mutex::*;
pub use task::*;

#[cfg(not(any(feature = "wasm32", feature = "wasm64")))]
compile_error!("Either feature \"wasm32\" or \"wasm64\" must be enabled");
#[cfg(all(feature = "wasm32", feature = "wasm64"))]
compile_error!("Features \"wasm32\" and \"wasm64\" cannot be enabled simultaneously");
