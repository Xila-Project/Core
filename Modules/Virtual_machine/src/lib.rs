#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod Custom_data;
mod Environment;
mod Error;
mod Instance;
mod Manager;
mod Module;
mod Registrable;
mod Runtime;

// Force linking of the ABI
#[allow(unused_imports)]
use ABI::*;

pub use wamr_rust_sdk::value::WasmValue;
pub use Custom_data::*;
pub use Environment::*;
pub use Error::*;
pub use Instance::*;
pub use Module::*;
pub use Registrable::*;
pub use Runtime::*;

pub type WASM_pointer_type = u32;
pub type WASM_usize_type = u32;
