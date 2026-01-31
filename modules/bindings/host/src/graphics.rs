use std::{
    cell::OnceCell,
    collections::{BTreeMap, btree_map::Entry},
    os::raw::c_void,
    ptr::null_mut,
};

pub use graphics::lvgl;
use task::block_on;

use task::TaskIdentifier;
use virtual_machine::{
    Environment, EnvironmentPointer, FunctionDescriptor, Registrable, WasmPointer, WasmUsize,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidArgumentsCount,
    InvalidPointer,
    NativePointerNotFound,
    WasmPointerNotFound,
    PointerTableFull,
    EnvironmentRetrievalFailed,
}

pub type Result<T> = core::result::Result<T, Error>;

mod generated_bindings {
    use super::{Error, PointerTable, Result, TaskIdentifier, lvgl::*};
    use virtual_machine::{Environment, WasmPointer, WasmUsize};

    unsafe fn convert_to_native_pointer<T>(
        environment: &Environment,
        pointer: WasmPointer,
    ) -> Result<*mut T> {
        unsafe {
            environment
                .convert_to_native_pointer(pointer)
                .ok_or(Error::InvalidPointer)
        }
    }

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

pub(crate) struct PointerTable {
    to_native_pointer: BTreeMap<usize, *mut c_void>,
    to_wasm_pointer: BTreeMap<*mut c_void, u16>,
}

impl PointerTable {
    pub fn new() -> Self {
        Self {
            to_native_pointer: BTreeMap::new(),
            to_wasm_pointer: BTreeMap::new(),
        }
    }

    const fn get_identifier(task: TaskIdentifier, identifier: u16) -> usize {
        (task.into_inner() as usize) << 32 | identifier as usize
    }

    pub fn insert(&mut self, task: TaskIdentifier, pointer: *mut c_void) -> Result<u16> {
        for i in u16::MIN..u16::MAX {
            let identifier = Self::get_identifier(task, i);

            match self.to_native_pointer.entry(identifier) {
                Entry::Vacant(entry) => {
                    entry.insert(pointer);
                    self.to_wasm_pointer.insert(pointer, i);
                    return Ok(i);
                }
                Entry::Occupied(entry_pointer) => {
                    if *entry_pointer.get() == pointer {
                        return Ok(i);
                    }
                }
            }
        }

        Err(Error::PointerTableFull)
    }

    pub fn get_native_pointer<T>(&self, task: TaskIdentifier, identifier: u16) -> Result<*mut T> {
        let identifier = Self::get_identifier(task, identifier);

        self.to_native_pointer
            .get(&identifier)
            .map(|pointer| *pointer as *mut T)
            .ok_or(Error::NativePointerNotFound)
    }

    pub fn get_wasm_pointer<T>(&self, pointer: *mut T) -> Result<u16> {
        self.to_wasm_pointer
            .get(&(pointer as *mut c_void))
            .cloned()
            .ok_or(Error::WasmPointerNotFound)
    }

    pub fn remove<T>(&mut self, task: TaskIdentifier, identifier: u16) -> Result<*mut T> {
        let identifier = Self::get_identifier(task, identifier);

        let pointer = self
            .to_native_pointer
            .remove(&identifier)
            .map(|pointer| pointer as *mut T)
            .ok_or(Error::NativePointerNotFound)?;

        self.to_wasm_pointer.remove(&(pointer as *mut _));

        Ok(pointer)
    }
}

static mut POINTER_TABLE: OnceCell<PointerTable> = OnceCell::new();

fn convert_argument(
    environment: &Environment,
    pointer_table: &mut PointerTable,
    task: TaskIdentifier,
    function: generated_bindings::FunctionCall,
    argument: WasmUsize,
    argument_index: usize,
) -> Result<usize> {
    if function.is_function_argument_pointer(argument_index) {
        let native_pointer = unsafe {
            environment
                .convert_to_native_pointer(argument as WasmPointer)
                .ok_or(Error::InvalidPointer)? as *mut c_void
        };

        if function.is_function_argument_lvgl_pointer(argument_index) {
            let native_pointer = native_pointer as *mut u16;

            let lvgl_pointer =
                pointer_table.get_native_pointer(task, unsafe { *native_pointer })? as *mut c_void;

            Ok(lvgl_pointer as usize)
        } else {
            Ok(native_pointer as usize)
        }
    } else {
        Ok(argument as usize)
    }
}

fn convert_result(
    environment: &Environment,
    pointer_table: &mut PointerTable,
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

pub struct CustomData<'a> {
    environment: Environment<'a>,
    pointer_table: &'a mut PointerTable,
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

        let _ = (*pointer_table_reference).get_or_init(PointerTable::new);

        let pointer_table_reference = (*pointer_table_reference).get_mut().unwrap();

        let task = environment
            .get_or_initialize_custom_data()
            .map_err(|_| Error::EnvironmentRetrievalFailed)?
            .get_task_identifier();

        let argument_0 = convert_argument(
            &environment,
            pointer_table_reference,
            task,
            function,
            argument_0,
            0,
        )?;
        let argument_1 = convert_argument(
            &environment,
            pointer_table_reference,
            task,
            function,
            argument_1,
            1,
        )?;
        let argument_2 = convert_argument(
            &environment,
            pointer_table_reference,
            task,
            function,
            argument_2,
            2,
        )?;
        let argument_3 = convert_argument(
            &environment,
            pointer_table_reference,
            task,
            function,
            argument_3,
            3,
        )?;
        let argument_4 = convert_argument(
            &environment,
            pointer_table_reference,
            task,
            function,
            argument_4,
            4,
        )?;
        let argument_5 = convert_argument(
            &environment,
            pointer_table_reference,
            task,
            function,
            argument_5,
            5,
        )?;
        let argument_6 = convert_argument(
            &environment,
            pointer_table_reference,
            task,
            function,
            argument_6,
            6,
        )?;
        let result_pointer = environment
            .convert_to_native_pointer(result_pointer)
            .ok_or(Error::InvalidPointer)? as *mut c_void;

        let custom_data = CustomData {
            environment: environment,
            pointer_table: pointer_table_reference,
        };

        generated_bindings::call_function(
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
            result_pointer,
            &custom_data as *const _ as *mut _,
        );

        if function.is_function_return_pointer() {
            let inner_pointer = *(result_pointer as *mut *mut c_void);

            if function.is_function_return_lvgl_pointer() {
                let result_pointer = result_pointer as *mut u16;

                *result_pointer = pointer_table_reference.insert(task, inner_pointer)?;
            } else {
                let result_pointer = result_pointer as *mut WasmPointer;

                *result_pointer = environment.convert_to_wasm_pointer(inner_pointer);
            }
        }

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
