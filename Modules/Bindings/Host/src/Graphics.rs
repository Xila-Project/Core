use std::{
    cell::OnceCell,
    collections::{btree_map::Entry, BTreeMap},
    os::raw::c_void,
};

use Futures::block_on;
pub use Graphics::LVGL;

use Task::Task_identifier_type;
use Virtual_machine::{
    Environment_pointer_type, Environment_type, Function_descriptor_type, Registrable_trait,
    WASM_pointer_type, WASM_usize_type,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error_type {
    Invalid_arguments_count,
    Invalid_pointer,
    Native_pointer_not_found,
    WASM_pointer_not_found,
    Pointer_table_full,
    Failed_to_get_environment,
}

pub type Result_type<T> = Result<T, Error_type>;

mod Generated_bindings {
    use super::{Error_type, Pointer_table_type, Result_type, Task_identifier_type, LVGL::*};
    use Virtual_machine::{Environment_type, WASM_pointer_type, WASM_usize_type};

    unsafe fn Convert_to_native_pointer<T>(
        environment: &Environment_type,
        pointer: WASM_pointer_type,
    ) -> Result_type<*mut T> {
        environment
            .Convert_to_native_pointer(pointer)
            .ok_or(Error_type::Invalid_pointer)
    }

    include!(concat!(env!("OUT_DIR"), "/Bindings.rs"));
}

pub struct Graphics_bindings;

impl Registrable_trait for Graphics_bindings {
    fn get_functions(&self) -> &[Function_descriptor_type] {
        &GRAPHICS_BINDINGS_FUNCTIONS
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn Is_XIP(&self) -> bool {
        true
    }

    fn Get_name(&self) -> &'static str {
        "Xila_graphics\0"
    }
}

pub(crate) struct Pointer_table_type {
    to_native_pointer: BTreeMap<usize, *mut c_void>,
    to_wasm_pointer: BTreeMap<*mut c_void, u16>,
}

impl Pointer_table_type {
    pub fn new() -> Self {
        Self {
            to_native_pointer: BTreeMap::new(),
            to_wasm_pointer: BTreeMap::new(),
        }
    }

    const fn Get_identifier(Task: Task_identifier_type, Identifier: u16) -> usize {
        (Task.Into_inner() as usize) << 32 | Identifier as usize
    }

    pub fn Insert(&mut self, Task: Task_identifier_type, Pointer: *mut c_void) -> Result_type<u16> {
        for i in u16::MIN..u16::MAX {
            let identifier = Self::Get_identifier(Task, i);

            match self.to_native_pointer.entry(identifier) {
                Entry::Vacant(entry) => {
                    entry.insert(Pointer);
                    self.to_wasm_pointer.insert(Pointer, i);
                    return Ok(i);
                }
                Entry::Occupied(Entry_pointer) => {
                    if *Entry_pointer.get() == Pointer {
                        return Ok(i);
                    }
                }
            }
        }

        Err(Error_type::Pointer_table_full)
    }

    pub fn Get_native_pointer<T>(
        &self,
        task: Task_identifier_type,
        identifier: u16,
    ) -> Result_type<*mut T> {
        let identifier = Self::Get_identifier(task, identifier);

        self.to_native_pointer
            .get(&identifier)
            .map(|pointer| *pointer as *mut T)
            .ok_or(Error_type::Native_pointer_not_found)
    }

    pub fn Get_wasm_pointer<T>(&self, Pointer: *mut T) -> Result_type<u16> {
        self.to_wasm_pointer
            .get(&(Pointer as *mut c_void))
            .cloned()
            .ok_or(Error_type::WASM_pointer_not_found)
    }

    pub fn Remove<T>(
        &mut self,
        task: Task_identifier_type,
        identifier: u16,
    ) -> Result_type<*mut T> {
        let identifier = Self::Get_identifier(task, identifier);

        let Pointer = self
            .to_native_pointer
            .remove(&identifier)
            .map(|pointer| pointer as *mut T)
            .ok_or(Error_type::Native_pointer_not_found)?;

        self.to_wasm_pointer.remove(&(Pointer as *mut _));

        Ok(Pointer)
    }
}

static mut POINTER_TABLE: OnceCell<Pointer_table_type> = OnceCell::new();

/// Call to graphics API
///
/// # Safety
///
/// This function is unsafe because it may dereference raw pointers (e.g. `Environment`, `Result` or `Arguments`).
/// The pointer must be valid and properly aligned (ensured by the virtual machine).
#[allow(clippy::too_many_arguments)]
pub unsafe fn call(
    environment: Environment_pointer_type,
    function: Generated_bindings::Function_calls_type,
    argument_0: WASM_usize_type,
    argument_1: WASM_usize_type,
    argument_2: WASM_usize_type,
    argument_3: WASM_usize_type,
    argument_4: WASM_usize_type,
    argument_5: WASM_usize_type,
    argument_6: WASM_usize_type,
    arguments_count: u8,
    result: WASM_pointer_type,
) {
    let Environment = Environment_type::from_raw_pointer(environment).unwrap();

    let Instance = Graphics::Get_instance();

    let _Lock = block_on(Instance.Lock());

    let Pointer_table_reference = &raw mut POINTER_TABLE;

    let _ = (*Pointer_table_reference).get_or_init(Pointer_table_type::new);

    let Pointer_table_reference = (*Pointer_table_reference).get_mut().unwrap();

    if let Err(Error) = Generated_bindings::Call_function(
        Environment,
        Pointer_table_reference,
        function,
        argument_0,
        argument_1,
        argument_2,
        argument_3,
        argument_4,
        argument_5,
        argument_6,
        arguments_count,
        result,
    ) {
        Log::Error!(
            "Error {Error:?} durring graphics call: {function:?} with arguments: {argument_0:x}, {argument_1:x}, {argument_2:x}, {argument_3:x}, {argument_4:x}, {argument_5:x}, {argument_6:x}",
        );
    }

    // Lock is automatically released here.
}

const GRAPHICS_BINDINGS_FUNCTIONS: [Function_descriptor_type; 1] = [Function_descriptor_type {
    name: "Xila_graphics_call",
    pointer: call as *mut _,
}];
