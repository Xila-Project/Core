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
        Environment: &Environment_type,
        Pointer: WASM_pointer_type,
    ) -> Result_type<*mut T> {
        Environment
            .Convert_to_native_pointer(Pointer)
            .ok_or(Error_type::Invalid_pointer)
    }

    include!(concat!(env!("OUT_DIR"), "/Bindings.rs"));
}

pub struct Graphics_bindings;

impl Registrable_trait for Graphics_bindings {
    fn Get_functions(&self) -> &[Function_descriptor_type] {
        &Graphics_bindings_functions
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
    To_native_pointer: BTreeMap<usize, *mut c_void>,
    To_wasm_pointer: BTreeMap<*mut c_void, u16>,
}

impl Pointer_table_type {
    pub fn New() -> Self {
        Self {
            To_native_pointer: BTreeMap::new(),
            To_wasm_pointer: BTreeMap::new(),
        }
    }

    const fn Get_identifier(Task: Task_identifier_type, Identifier: u16) -> usize {
        (Task.Into_inner() as usize) << 32 | Identifier as usize
    }

    pub fn Insert(&mut self, Task: Task_identifier_type, Pointer: *mut c_void) -> Result_type<u16> {
        for i in u16::MIN..u16::MAX {
            let Identifier = Self::Get_identifier(Task, i);

            match self.To_native_pointer.entry(Identifier) {
                Entry::Vacant(entry) => {
                    entry.insert(Pointer);
                    self.To_wasm_pointer.insert(Pointer, i);
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
        Task: Task_identifier_type,
        Identifier: u16,
    ) -> Result_type<*mut T> {
        let Identifier = Self::Get_identifier(Task, Identifier);

        self.To_native_pointer
            .get(&Identifier)
            .map(|Pointer| *Pointer as *mut T)
            .ok_or(Error_type::Native_pointer_not_found)
    }

    pub fn Get_wasm_pointer<T>(&self, Pointer: *mut T) -> Result_type<u16> {
        self.To_wasm_pointer
            .get(&(Pointer as *mut c_void))
            .cloned()
            .ok_or(Error_type::WASM_pointer_not_found)
    }

    pub fn Remove<T>(
        &mut self,
        Task: Task_identifier_type,
        Identifier: u16,
    ) -> Result_type<*mut T> {
        let Identifier = Self::Get_identifier(Task, Identifier);

        let Pointer = self
            .To_native_pointer
            .remove(&Identifier)
            .map(|Pointer| Pointer as *mut T)
            .ok_or(Error_type::Native_pointer_not_found)?;

        self.To_wasm_pointer.remove(&(Pointer as *mut _));

        Ok(Pointer)
    }
}

static mut Pointer_table: OnceCell<Pointer_table_type> = OnceCell::new();

/// Call to graphics API
///
/// # Safety
///
/// This function is unsafe because it may dereference raw pointers (e.g. `Environment`, `Result` or `Arguments`).
/// The pointer must be valid and properly aligned (ensured by the virtual machine).
#[allow(clippy::too_many_arguments)]
pub unsafe fn Call(
    Environment: Environment_pointer_type,
    Function: Generated_bindings::Function_calls_type,
    Argument_0: WASM_usize_type,
    Argument_1: WASM_usize_type,
    Argument_2: WASM_usize_type,
    Argument_3: WASM_usize_type,
    Argument_4: WASM_usize_type,
    Argument_5: WASM_usize_type,
    Argument_6: WASM_usize_type,
    Arguments_count: u8,
    Result: WASM_pointer_type,
) {
    let Environment = Environment_type::From_raw_pointer(Environment).unwrap();

    let Instance = Graphics::Get_instance();

    let _Lock = block_on(Instance.Lock());

    let Pointer_table_reference = &raw mut Pointer_table;

    let _ = (*Pointer_table_reference).get_or_init(Pointer_table_type::New);

    let Pointer_table_reference = (*Pointer_table_reference).get_mut().unwrap();

    if let Err(Error) = Generated_bindings::Call_function(
        Environment,
        Pointer_table_reference,
        Function,
        Argument_0,
        Argument_1,
        Argument_2,
        Argument_3,
        Argument_4,
        Argument_5,
        Argument_6,
        Arguments_count,
        Result,
    ) {
        Log::Error!(
            "Error {:?} durring graphics call: {:?} with arguments: {:x}, {:x}, {:x}, {:x}, {:x}, {:x}, {:x}",
            Error,
            Function,
            Argument_0,
            Argument_1,
            Argument_2,
            Argument_3,
            Argument_4,
            Argument_5,
            Argument_6,
        );
    }

    // Lock is automatically released here.
}

const Graphics_bindings_functions: [Function_descriptor_type; 1] = [Function_descriptor_type {
    Name: "Xila_graphics_call",
    Pointer: Call as *mut _,
}];
