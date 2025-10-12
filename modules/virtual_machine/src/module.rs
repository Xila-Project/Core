use core::{ffi::CStr, ptr::null_mut};

use alloc::vec::Vec;
use file_system::UniqueFileIdentifier;
use wamr_rust_sdk::{module, sys::wasm_runtime_set_wasi_args_ex};

use crate::{Error, Result, runtime::Runtime};

pub struct Module<'runtime> {
    module: module::Module<'runtime>,
    _environment_variables_raw: Vec<*const i8>,
}

unsafe impl Send for Module<'_> {}

const DIRECTORY_PATHS: [&CStr; 1] = [c"/"];
const DIRECTORY_PATHS_RAW: [*const i8; 1] = [DIRECTORY_PATHS[0].as_ptr()];

impl<'runtime> Module<'runtime> {
    pub async fn from_buffer(
        runtime: &'runtime Runtime,
        buffer: Vec<u8>,
        name: &str,
        standard_in: UniqueFileIdentifier,
        standard_out: UniqueFileIdentifier,
        standard_error: UniqueFileIdentifier,
    ) -> Result<Self> {
        // - Environment variables.
        let task_instance = task::get_instance();

        let task = task_instance.get_current_task_identifier().await;
        let mut environment_variables_raw: Vec<*const i8> = task_instance
            .get_environment_variables(task)
            .await
            .map_err(Error::FailedToGetTaskInformations)?
            .into_iter()
            .map(|x| x.get_raw().as_ptr())
            .collect();

        let environment_variables_raw_pointer = environment_variables_raw.as_mut_ptr();

        let environment_variables_length = environment_variables_raw.len();

        // - Create the module.
        let module = Self {
            module: module::Module::from_vec(runtime.get_inner_reference(), buffer, name)?,
            _environment_variables_raw: environment_variables_raw,
        };

        let standard_in = standard_in.into_inner() as u64;
        let standard_out = standard_out.into_inner() as u64;
        let standard_error = standard_error.into_inner() as u64;

        // - Set WASI arguments.
        unsafe {
            wasm_runtime_set_wasi_args_ex(
                module.module.get_inner_module(),
                DIRECTORY_PATHS_RAW.as_ptr() as *mut *const i8,
                DIRECTORY_PATHS_RAW.len() as u32,
                null_mut(),
                0,
                environment_variables_raw_pointer,
                environment_variables_length as u32,
                null_mut(),
                0,
                u64::cast_signed(standard_in),
                u64::cast_signed(standard_out),
                u64::cast_signed(standard_error),
            )
        }

        Ok(module)
    }

    pub(crate) fn get_inner_reference(&'_ self) -> &'_ module::Module<'_> {
        &self.module
    }
}
