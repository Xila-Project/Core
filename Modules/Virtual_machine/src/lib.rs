#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod Data;
mod Error;
mod Environment;
mod Instance;
mod Module;
mod Registrable;
mod Runtime;

pub use wamr_rust_sdk::value::WasmValue;
pub use Data::*;
pub use Error::*;
pub use Environment::*;
pub use Instance::*;
pub use Module::*;
pub use Registrable::*;
pub use Runtime::*;

pub type WASM_pointer = u32;
pub type WASM_usize = u32;

pub fn Instantiate_test_environment(
    Binary_buffer: &[u8],
    Registrable: impl Registrable_trait,
    User_data: &Data_type,
) -> (Runtime_type, Module_type, Instance_type) {
    let Runtime = Runtime_type::Builder()
        .Register(Registrable)
        .Build()
        .unwrap();

    let Module = Module_type::From_buffer(&Runtime, Binary_buffer, "main").unwrap();

    let Instance = Instance_type::New(&Runtime, &Module, 1024 * 4, User_data).unwrap();

    (Runtime, Module, Instance)
}
