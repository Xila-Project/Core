#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod Data;
mod Environment;
mod Error;
mod Instance;
mod Module;
mod Registrable;
mod Runtime;

// For
#[allow(unused_imports)]
use ABI::*;

use std::{
    io::{stderr, stdin, stdout, Read, Write},
    sync::Arc,
};

pub use wamr_rust_sdk::value::WasmValue;
pub use Data::*;
pub use Environment::*;
pub use Error::*;
use File_system::{Device_trait, Device_type, Size_type, Virtual_file_system_type};
pub use Instance::*;
pub use Module::*;
pub use Registrable::*;
pub use Runtime::*;

pub type WASM_pointer = u32;
pub type WASM_usize = u32;

struct Standard_in_device_type;

impl Device_trait for Standard_in_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<Size_type> {
        #[allow(clippy::unused_io_amount)]
        stdin().read(Buffer).unwrap();

        Ok(Size_type::New(Buffer.len() as u64))
    }

    fn Write(&self, _: &[u8]) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Get_size(&self) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }

    fn Is_a_terminal(&self) -> bool {
        true
    }
}

struct Standard_out_device_type;

impl Device_trait for Standard_out_device_type {
    fn Read(&self, _: &mut [u8]) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(stdout().write(Buffer)? as u64))
    }

    fn Get_size(&self) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(stdout().flush()?)
    }

    fn Is_a_terminal(&self) -> bool {
        true
    }
}

struct Standard_error_device_type;

impl Device_trait for Standard_error_device_type {
    fn Read(&self, _: &mut [u8]) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(stderr().write(Buffer)? as u64))
    }

    fn Get_size(&self) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(stderr().flush()?)
    }

    fn Is_a_terminal(&self) -> bool {
        true
    }
}

pub fn Instantiate_test_environment(
    Binary_buffer: &[u8],
    Registrable: impl Registrable_trait,
    User_data: &Data_type,
    Task_manager: &Task::Manager_type,
    Virtual_file_system: &Virtual_file_system_type,
) -> (Runtime_type, Module_type, Instance_type) {
    let Runtime = Runtime_type::Builder()
        .Register(Registrable)
        .Build()
        .unwrap();

    let Module = Module_type::From_buffer(&Runtime, Binary_buffer, "main").unwrap();

    let Task = Task_manager.Get_current_task_identifier().unwrap();

    Virtual_file_system
        .Mount_device(
            Task,
            "/stdin",
            Device_type::New(Arc::new(Standard_in_device_type)),
            false,
        )
        .unwrap();

    Virtual_file_system
        .Mount_device(
            Task,
            "/stdout",
            Device_type::New(Arc::new(Standard_out_device_type)),
            false,
        )
        .unwrap();

    Virtual_file_system
        .Mount_device(
            Task,
            "/stderr",
            Device_type::New(Arc::new(Standard_error_device_type)),
            false,
        )
        .unwrap();

    let Stdin = Virtual_file_system
        .Open(&"/stdin", File_system::Mode_type::Read_only.into(), Task)
        .expect("Failed to open stdin");
    let Stdout = Virtual_file_system
        .Open(&"/stdout", File_system::Mode_type::Write_only.into(), Task)
        .expect("Failed to open stdout");
    let Stderr = Virtual_file_system
        .Open(&"/stderr", File_system::Mode_type::Write_only.into(), Task)
        .expect("Failed to open stderr");

    let (Stdin, Stdout, Stderr) = Virtual_file_system
        .Create_new_task_standard_io(Stdin, Stderr, Stdout, Task, Task, false)
        .unwrap();

    let Instance = Instance_type::New(
        &Runtime,
        &Module,
        1024 * 4,
        User_data,
        Stdin,
        Stdout,
        Stderr,
    )
    .expect("Failed to instantiate module");

    (Runtime, Module, Instance)
}
