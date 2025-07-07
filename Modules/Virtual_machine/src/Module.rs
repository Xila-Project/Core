use core::{ffi::CStr, ptr::null_mut};

use alloc::vec::Vec;
use wamr_rust_sdk::{module::Module, sys::wasm_runtime_set_wasi_args_ex};
use File_system::Unique_file_identifier_type;

use crate::{Error_type, Result_type, Runtime::Runtime_type};

pub struct Module_type<'runtime> {
    module: Module<'runtime>,
    _environment_variables_raw: Vec<*const i8>,
}

unsafe impl Send for Module_type<'_> {}

const DIRECTORY_PATHS: [&CStr; 1] = [c"/"];
const DIRECTORY_PATHS_RAW: [*const i8; 1] = [DIRECTORY_PATHS[0].as_ptr()];

impl<'runtime> Module_type<'runtime> {
    pub async fn From_buffer(
        runtime: &'runtime Runtime_type,
        buffer: Vec<u8>,
        name: &str,
        standard_in: Unique_file_identifier_type,
        standard_out: Unique_file_identifier_type,
        standard_error: Unique_file_identifier_type,
    ) -> Result_type<Self> {
        // - Environment variables.
        let Task_instance = Task::Get_instance();

        let Task = Task_instance.Get_current_task_identifier().await;
        let mut environment_variables_raw: Vec<*const i8> = Task_instance
            .Get_environment_variables(Task)
            .await
            .map_err(Error_type::Failed_to_get_task_informations)?
            .into_iter()
            .map(|x| x.Get_raw().as_ptr())
            .collect();

        let Environment_variables_raw_pointer = environment_variables_raw.as_mut_ptr();

        let Environment_variables_length = environment_variables_raw.len();

        // - Create the module.
        let Module = Module_type {
            module: Module::from_vec(runtime.Get_inner_reference(), buffer, name)?,
            _environment_variables_raw: environment_variables_raw,
        };

        let Standard_in = standard_in.Into_inner() as u64;
        let standard_out = standard_out.Into_inner() as u64;
        let standard_error = standard_error.Into_inner() as u64;

        // - Set WASI arguments.
        unsafe {
            wasm_runtime_set_wasi_args_ex(
                Module.module.get_inner_module(),
                DIRECTORY_PATHS_RAW.as_ptr() as *mut *const i8,
                DIRECTORY_PATHS_RAW.len() as u32,
                null_mut(),
                0,
                Environment_variables_raw_pointer,
                Environment_variables_length as u32,
                null_mut(),
                0,
                u64::cast_signed(Standard_in),
                u64::cast_signed(standard_out),
                u64::cast_signed(standard_error),
            )
        }

        Ok(Module)
    }

    pub(crate) fn Get_inner_reference(&self) -> &Module {
        &self.module
    }
}
