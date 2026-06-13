use core::{
    cmp::max,
    mem::transmute,
    ops::{Deref, DerefMut},
    ptr,
    task::Poll,
};

use crate::host::virtual_machine::{Environment, Error, Instance, Result};

use alloc::{string::ToString, vec, vec::Vec};
use wamr_rust_sdk::{
    sys::{
        WASMFunctionInstanceCommon, wasm_func_get_param_count, wasm_func_get_result_count,
        wasm_func_get_result_types, wasm_runtime_call_wasm, wasm_runtime_resume_wasm,
        wasm_valkind_enum_WASM_EXTERNREF, wasm_valkind_enum_WASM_F32, wasm_valkind_enum_WASM_F64,
        wasm_valkind_enum_WASM_FUNCREF, wasm_valkind_enum_WASM_I32, wasm_valkind_enum_WASM_I64,
        wasm_valkind_enum_WASM_V128,
    },
    value::WasmValue,
};
use wasm_abi_bindings::WasmUsize;

pub struct Function<'instance> {
    function: wamr_rust_sdk::function::Function<'instance>,
    arguments: Option<Vec<WasmUsize>>,
}

impl<'instance> Function<'instance> {
    pub fn find_exported(instance: &'instance Instance<'instance>, name: &str) -> Result<Self> {
        let function =
            wamr_rust_sdk::function::Function::find_export_func(instance.as_ref(), name)?;

        Ok(Self {
            function,
            arguments: None,
        })
    }

    pub fn get_inner_function(&self) -> *mut WASMFunctionInstanceCommon {
        unsafe {
            let funciton = ptr::read(&self.function);

            transmute::<wamr_rust_sdk::function::Function<'instance>, *mut WASMFunctionInstanceCommon>(
                funciton,
            )
        }
    }

    pub fn call(
        &mut self,
        instance: &Instance<'instance>,
        environment: &mut Environment<'instance>,
        arguments: &Vec<WasmValue>,
    ) -> Poll<Result<Vec<WasmValue>>> {
        let resume = self.arguments.is_some();

        let inner_function = self.get_inner_function();

        let arguments = self
            .arguments
            .get_or_insert(self.convert_arguments(instance, arguments)?);

        let call_ok = if resume {
            unsafe { wasm_runtime_resume_wasm(environment.as_raw_pointer()) }
        } else {
            unsafe {
                wasm_runtime_call_wasm(
                    environment.as_raw_pointer(),
                    inner_function,
                    arguments.len() as u32,
                    arguments.as_mut_ptr(),
                )
            }
        };

        if call_ok {
            let arguments_copy = arguments.clone();
            let results = self.parse_results(instance, &arguments_copy)?;

            return Poll::Ready(Ok(results));
        }

        let exception = instance.get_current_exception().ok_or(Error::Execution(
            "failed to get exception after failed function call".into(),
        ))?;

        if !exception.contains("instruction limit exceeded") {
            return Poll::Ready(Err(Error::Execution(exception.to_string())));
        }

        instance.clear_exception();

        Poll::Pending
    }

    fn convert_arguments(
        &self,
        instance: &Instance<'instance>,
        arguments: &Vec<WasmValue>,
    ) -> Result<Vec<WasmUsize>> {
        let (param_count, result_count) = unsafe {
            (
                wasm_func_get_param_count(self.get_inner_function(), instance.as_raw_pointer()),
                wasm_func_get_result_count(self.get_inner_function(), instance.as_raw_pointer()),
            )
        };

        if param_count != arguments.len() as u32 {
            return Err(Error::Execution("invalid parameters".into()));
        }

        let param_cell_count: usize = arguments
            .iter()
            .map(|parameter| parameter.encode().len())
            .sum();

        let result_cell_count = result_count as usize * core::mem::size_of::<WasmUsize>();

        let capacity = max(1, max(param_cell_count, result_cell_count));

        let mut argument_cells = Vec::with_capacity(capacity);

        for argument in arguments {
            argument_cells.extend(argument.encode());
        }

        Ok(argument_cells)
    }

    #[allow(non_upper_case_globals)]
    fn parse_results(
        &self,
        instance: &Instance<'instance>,
        argv: &[u32],
    ) -> Result<Vec<WasmValue>> {
        let result_count = unsafe {
            wasm_func_get_result_count(self.get_inner_function(), instance.as_raw_pointer())
        };
        if result_count == 0 {
            return Ok(vec![WasmValue::Void]);
        }

        let mut result_types = vec![0u8; result_count as usize];
        unsafe {
            wasm_func_get_result_types(
                self.get_inner_function(),
                instance.as_raw_pointer(),
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
}

impl<'instance> Deref for Function<'instance> {
    type Target = wamr_rust_sdk::function::Function<'instance>;

    fn deref(&self) -> &Self::Target {
        &self.function
    }
}

impl<'instance> DerefMut for Function<'instance> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.function
    }
}
