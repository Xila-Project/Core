mod additionnal;
mod cast;
mod error;
mod translation_map;

use crate::host::bindings::graphics::{
    translation::translate_arguments, translation_map::TranslationMap,
};
use crate::host::virtual_machine::{
    Environment, EnvironmentPointer, FunctionDescriptor, Registrable, WasmPointer, WasmUsize,
};
use core::ffi::c_void;
use xila::{
    graphics, log,
    synchronization::once_lock::OnceLock,
    task::{TaskIdentifier, block_on},
};

pub use error::{Error, Result};
pub use xila::graphics::lvgl;

mod generated_bindings {
    use super::{
        Error, Result, TaskIdentifier,
        cast::{FromUsize, ToUsize},
        lvgl::*,
    };

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
pub struct GraphicsBindings;

impl Registrable for GraphicsBindings {
    fn get_functions(&self) -> &[FunctionDescriptor] {
        &GRAPHICS_BINDINGS_FUNCTIONS
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn is_xip(&self) -> bool {
        true
    }

    fn get_name(&self) -> &'static str {
        "xila_graphics\0"
    }
}

static mut POINTER_TABLE: OnceLock<TranslationMap> = OnceLock::new();

fn convert_argument_custom(
    _environment: &Environment,
    _pointer_table: &mut TranslationMap,
    _task: TaskIdentifier,
    _function: generated_bindings::FunctionCall,
    _argument_index: usize,
    _argument: WasmUsize,
) -> Result<usize> {
    // No custom casting implemented yet
    Ok(_argument as usize)
}

fn convert_result(
    environment: &Environment,
    pointer_table: &mut TranslationMap,
    task: TaskIdentifier,
    function: generated_bindings::FunctionCall,
    result_pointer: WasmPointer,
) -> Result<()> {
    if function.is_function_return_pointer() {
        let native_pointer = unsafe {
            environment
                .convert_to_native_pointer(result_pointer)
                .ok_or(Error::InvalidPointer)? as *mut c_void
        };

        if function.is_function_return_lvgl_pointer() {
            let lvgl_pointer = native_pointer as *mut u16;

            let wasm_identifier = pointer_table.get_wasm_pointer(
                pointer_table.get_native_pointer::<c_void>(task, unsafe { *lvgl_pointer })?,
            )?;

            unsafe {
                *lvgl_pointer = wasm_identifier;
            }
        }
    }

    Ok(())
}

/// Call to graphics API
///
/// # Safety
///
/// This function is unsafe because it may dereference raw pointers (e.g. `Environment`, `Result` or `Arguments`).
/// The pointer must be valid and properly aligned (ensured by the virtual machine).
#[allow(clippy::too_many_arguments)]
unsafe fn call_inner(
    environment: EnvironmentPointer,
    function: generated_bindings::FunctionCall,
    argument_0: WasmUsize,
    argument_1: WasmUsize,
    argument_2: WasmUsize,
    argument_3: WasmUsize,
    argument_4: WasmUsize,
    argument_5: WasmUsize,
    argument_6: WasmUsize,
    arguments_count: u8,
    result_pointer: WasmPointer,
) -> Result<()> {
    unsafe {
        let environment = Environment::from_raw_pointer(environment).unwrap();

        let instance = graphics::get_instance();

        let _lock = block_on(instance.lock());

        let pointer_table_reference = &raw mut POINTER_TABLE;

        let translation_map = (*pointer_table_reference).get_or_init(TranslationMap::new);

        let translation_map = unsafe { &mut *(translation_map as *const _ as *mut _) };

        let task = environment
            .get_or_initialize_custom_data()
            .map_err(|_| Error::EnvironmentRetrievalFailed)?
            .get_task_identifier();

        let result_pointer = environment
            .convert_to_native_pointer(result_pointer)
            .ok_or(Error::InvalidPointer)? as *mut c_void;

        let result = generated_bindings::call_function(
            environment,
            translation_map,
            task,
            function,
            argument_0,
            argument_1,
            argument_2,
            argument_3,
            argument_4,
            argument_5,
            argument_6,
            arguments_count,
        )?;

        convert_result(
            &environment,
            translation_map,
            task,
            function,
            result_pointer as WasmPointer,
        )?;

        Ok(())

        // Lock is automatically released here.
    }
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn call(
    environment: EnvironmentPointer,
    function: generated_bindings::FunctionCall,
    argument_0: WasmUsize,
    argument_1: WasmUsize,
    argument_2: WasmUsize,
    argument_3: WasmUsize,
    argument_4: WasmUsize,
    argument_5: WasmUsize,
    argument_6: WasmUsize,
    arguments_count: u8,
    result_pointer: WasmPointer,
) -> i32 {
    let result = unsafe {
        call_inner(
            environment,
            function,
            argument_0,
            argument_1,
            argument_2,
            argument_3,
            argument_4,
            argument_5,
            argument_6,
            arguments_count,
            result_pointer,
        )
    };

    match result {
        Ok(_) => 0,
        Err(error) => {
            log::error!(
                "Error {error:?} durring graphics call: {function:?} with arguments: {argument_0:x}, {argument_1:x}, {argument_2:x}, {argument_3:x}, {argument_4:x}, {argument_5:x}, {argument_6:x}, count: {arguments_count}, result pointer: {result_pointer:x}"
            );
            error as i32
        }
    }
}

const GRAPHICS_BINDINGS_FUNCTIONS: [FunctionDescriptor; 1] = [FunctionDescriptor {
    name: "xila_graphics_call",
    pointer: call as *mut _,
}];
