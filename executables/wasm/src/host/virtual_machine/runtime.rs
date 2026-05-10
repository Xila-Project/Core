//! WASM Runtime management and configuration.
//!
//! This module provides a wrapper around the WAMR runtime with a builder pattern
//! for configuring and creating runtime instances with registered host functions.

use crate::host::virtual_machine::{Error, Instance, Module, Registrable, Result};
use alloc::{ffi::CString, string::String, vec, vec::Vec};
use core::{cmp, ffi::CStr};
use wamr_rust_sdk::{
    sys::{
        wasm_exec_env_t, wasm_func_get_param_count, wasm_func_get_result_count,
        wasm_func_get_result_types, wasm_function_inst_t, wasm_runtime_call_wasm,
        wasm_runtime_clear_exception, wasm_runtime_create_exec_env, wasm_runtime_destroy_exec_env,
        wasm_runtime_get_exception, wasm_runtime_lookup_function, wasm_runtime_resume_wasm,
        wasm_runtime_set_instruction_count_limit, wasm_valkind_enum_WASM_EXTERNREF,
        wasm_valkind_enum_WASM_F32, wasm_valkind_enum_WASM_F64, wasm_valkind_enum_WASM_FUNCREF,
        wasm_valkind_enum_WASM_I32, wasm_valkind_enum_WASM_I64, wasm_valkind_enum_WASM_V128,
    },
    value::WasmValue,
};
use xila::{
    abi_context::{self, FileIdentifier},
    log,
    task::{self, TaskIdentifier, yield_now},
    virtual_file_system::File,
};

pub struct Runtime(wamr_rust_sdk::runtime::Runtime);

unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

impl Runtime {
    pub fn new<'a>(registrables: impl IntoIterator<Item = &'a dyn Registrable>) -> Result<Self> {
        let mut runtime_builder = wamr_rust_sdk::runtime::Runtime::builder()
            .use_system_allocator()
            .run_as_interpreter();

        for registrable in registrables {
            for function_descriptor in registrable.get_functions() {
                runtime_builder = runtime_builder
                    .register_host_function(function_descriptor.name, function_descriptor.pointer);
            }
        }

        let runtime = runtime_builder.build()?;

        Ok(Self(runtime))
    }

    pub fn get_inner_reference(&self) -> &wamr_rust_sdk::runtime::Runtime {
        &self.0
    }

    fn current_exception_string(instance: &wamr_rust_sdk::instance::Instance<'_>) -> String {
        unsafe {
            let exception = wasm_runtime_get_exception(instance.get_inner_instance());
            if exception.is_null() {
                return String::new();
            }

            CStr::from_ptr(exception).to_string_lossy().into_owned()
        }
    }

    #[allow(non_upper_case_globals)]
    fn parse_results(
        function: wasm_function_inst_t,
        instance: &wamr_rust_sdk::instance::Instance<'_>,
        argv: &[u32],
    ) -> Result<Vec<WasmValue>> {
        let result_count =
            unsafe { wasm_func_get_result_count(function, instance.get_inner_instance()) };
        if result_count == 0 {
            return Ok(vec![WasmValue::Void]);
        }

        let mut result_types = vec![0u8; result_count as usize];
        unsafe {
            wasm_func_get_result_types(
                function,
                instance.get_inner_instance(),
                result_types.as_mut_ptr(),
            );
        }

        let mut results = Vec::with_capacity(result_types.len());
        let mut index: usize = 0;

        for result_type in result_types.iter() {
            match *result_type as u32 {
                wasm_valkind_enum_WASM_I32
                | wasm_valkind_enum_WASM_FUNCREF
                | wasm_valkind_enum_WASM_EXTERNREF => {
                    if index + 1 > argv.len() {
                        return Err(Error::Execution("invalid return buffer".into()));
                    }
                    results.push(WasmValue::decode_to_i32(&argv[index..index + 1]));
                    index += 1;
                }
                wasm_valkind_enum_WASM_I64 => {
                    if index + 2 > argv.len() {
                        return Err(Error::Execution("invalid return buffer".into()));
                    }
                    results.push(WasmValue::decode_to_i64(&argv[index..index + 2]));
                    index += 2;
                }
                wasm_valkind_enum_WASM_F32 => {
                    if index + 1 > argv.len() {
                        return Err(Error::Execution("invalid return buffer".into()));
                    }
                    results.push(WasmValue::decode_to_f32(&argv[index..index + 1]));
                    index += 1;
                }
                wasm_valkind_enum_WASM_F64 => {
                    if index + 2 > argv.len() {
                        return Err(Error::Execution("invalid return buffer".into()));
                    }
                    results.push(WasmValue::decode_to_f64(&argv[index..index + 2]));
                    index += 2;
                }
                wasm_valkind_enum_WASM_V128 => {
                    if index + 4 > argv.len() {
                        return Err(Error::Execution("invalid return buffer".into()));
                    }
                    results.push(WasmValue::decode_to_v128(&argv[index..index + 4]));
                    index += 4;
                }
                _ => return Err(Error::NotImplemented),
            }
        }

        Ok(results)
    }

    /// Execute a WASM module with the specified I/O configuration.
    ///
    /// This is the main entry point for executing WASM modules. It creates a new
    /// module instance, sets up the execution environment with proper I/O redirection,
    /// and calls the module's main function.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(
        &'static self,
        name: &str,
        buffer: Vec<u8>,
        stack_size: usize,
        (standard_in, standard_out, standard_error): (File, File, File),
        function_name: &str,
        function_arguments: Vec<WasmValue>,
        task: TaskIdentifier,
        instruction_limit: u32,
    ) -> Result<Vec<WasmValue>> {
        let abi_context = abi_context::get_instance();

        abi_context
            .call_abi(async || {
                let standard_in = abi_context
                    .insert_file(
                        task,
                        standard_in.into_synchronous_file(),
                        Some(FileIdentifier::STANDARD_IN),
                    )
                    .ok_or(Error::FailedToRegisterFileContext)?;
                let standard_out = abi_context
                    .insert_file(
                        task,
                        standard_out.into_synchronous_file(),
                        Some(FileIdentifier::STANDARD_OUT),
                    )
                    .ok_or(Error::FailedToRegisterFileContext)?;
                let standard_error = abi_context
                    .insert_file(
                        task,
                        standard_error.into_synchronous_file(),
                        Some(FileIdentifier::STANDARD_ERROR),
                    )
                    .ok_or(Error::FailedToRegisterFileContext)?;

                let module = Module::from_buffer(
                    self,
                    buffer,
                    name,
                    standard_in,
                    standard_out,
                    standard_error,
                )
                .await?;

                let instance = Instance::new(self, &module, stack_size as _)?;

                let function_name =
                    CString::new(function_name).map_err(|_| Error::InvalidUtf8String)?;
                let function = unsafe {
                    wasm_runtime_lookup_function(
                        instance.get_inner_reference().get_inner_instance(),
                        function_name.as_ptr(),
                    )
                };

                if function.is_null() {
                    return Err(Error::FunctionNotFound);
                }

                let param_count = unsafe {
                    wasm_func_get_param_count(
                        function,
                        instance.get_inner_reference().get_inner_instance(),
                    )
                };

                if param_count > function_arguments.len() as u32 {
                    return Err(Error::Execution("invalid parameters".into()));
                }

                let result_count = unsafe {
                    wasm_func_get_result_count(
                        function,
                        instance.get_inner_reference().get_inner_instance(),
                    )
                };

                let mut argv = Vec::new();
                let mut param_cell_count = 0usize;
                for parameter in &function_arguments {
                    let encoded = parameter.encode();
                    param_cell_count += encoded.len();
                    argv.extend(encoded);
                }

                let result_cell_count = result_count as usize * 4;
                let capacity = cmp::max(1, cmp::max(param_cell_count, result_cell_count));
                argv.resize(capacity, 0);

                let execution_environment: wasm_exec_env_t = unsafe {
                    wasm_runtime_create_exec_env(
                        instance.get_inner_reference().get_inner_instance(),
                        stack_size as _,
                    )
                };

                if execution_environment.is_null() {
                    return Err(Error::Execution(
                        "failed to create WASM execution environment".into(),
                    ));
                }

                let instruction_limit = instruction_limit.max(1);

                unsafe {
                    wasm_runtime_set_instruction_count_limit(
                        execution_environment,
                        instruction_limit as _,
                    );
                }

                let mut resume = false;
                let execution_result = loop {
                    let call_ok = unsafe {
                        if resume {
                            wasm_runtime_resume_wasm(execution_environment)
                        } else {
                            wasm_runtime_call_wasm(
                                execution_environment,
                                function,
                                param_count,
                                argv.as_mut_ptr(),
                            )
                        }
                    };

                    if call_ok {
                        break Self::parse_results(function, instance.get_inner_reference(), &argv);
                    }

                    let exception = Self::current_exception_string(instance.get_inner_reference());

                    if exception.contains("instruction limit exceeded") {
                        if let Some(requested_sleep) =
                            abi_context::get_instance().take_sleep_request(task)
                        {
                            if !requested_sleep.is_zero() {
                                log::information!(
                                    "Task {} requested sleep for {:?} ms",
                                    task.into_inner(),
                                    requested_sleep
                                );
                                task::sleep(requested_sleep).await;
                            } else {
                                yield_now().await;
                            }
                        } else {
                            yield_now().await;
                        }

                        unsafe {
                            wasm_runtime_clear_exception(
                                instance.get_inner_reference().get_inner_instance(),
                            );
                            wasm_runtime_set_instruction_count_limit(
                                execution_environment,
                                instruction_limit as _,
                            );
                        }

                        resume = true;

                        continue;
                    }

                    let _ = abi_context::get_instance().take_sleep_request(task);

                    break if exception.is_empty() {
                        Err(Error::Execution("WASM execution failed".into()))
                    } else {
                        Err(Error::Execution(exception))
                    };
                };

                unsafe {
                    wasm_runtime_destroy_exec_env(execution_environment);
                }

                execution_result
            })
            .await
    }
}
