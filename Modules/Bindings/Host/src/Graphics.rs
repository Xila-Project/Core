use std::{
    cell::OnceCell,
    collections::{btree_map::Entry, BTreeMap},
    os::raw::c_void,
};

pub use Graphics::lvgl;

use Task::Task_identifier_type;
use Virtual_machine::{
    Environment_pointer_type, Environment_type, Function_descriptor_type, Registrable_trait,
    WASM_pointer_type, WASM_usize_type,
};

mod Generated_bindings {
    use super::{lvgl::*, Pointer_table_type, Task_identifier_type};
    use Virtual_machine::{Environment_type, WASM_pointer_type, WASM_usize_type};

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
    Pointers_table: BTreeMap<usize, *mut c_void>,
}

impl Pointer_table_type {
    pub fn New() -> Self {
        Self {
            Pointers_table: BTreeMap::new(),
        }
    }

    const fn Get_identifier(Task: Task_identifier_type, Identifier: u16) -> usize {
        (Task.Into_inner() as usize) << 32 | Identifier as usize
    }

    pub fn Insert(&mut self, Task: Task_identifier_type, Pointer: *mut c_void) -> Option<u16> {
        for i in u16::MIN..u16::MAX {
            let Identifier = Self::Get_identifier(Task, i);

            match self.Pointers_table.entry(Identifier) {
                Entry::Vacant(entry) => {
                    entry.insert(Pointer);
                    return Some(i);
                }
                Entry::Occupied(Entry_pointer) => {
                    if *Entry_pointer.get() == Pointer {
                        return Some(i);
                    }
                }
            }
        }

        None
    }

    pub fn Get<T>(&self, Task: Task_identifier_type, Identifier: u16) -> Option<*mut T> {
        let Identifier = Self::Get_identifier(Task, Identifier);

        self.Pointers_table
            .get(&Identifier)
            .map(|Pointer| *Pointer as *mut T)
    }

    pub fn Remove<T>(&mut self, Task: Task_identifier_type, Identifier: u16) -> Option<*mut T> {
        let Identifier = Self::Get_identifier(Task, Identifier);

        self.Pointers_table
            .remove(&Identifier)
            .map(|Pointer| Pointer as *mut T)
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

    let Lock = Instance.Lock();

    if Lock.is_ok() {
        let _ = Pointer_table.get_or_init(Pointer_table_type::New);

        let Pointer_table_reference = Pointer_table.get_mut().unwrap();

        Generated_bindings::Call_function(
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
        );
    }

    // Lock is automatically released here.
}

const Graphics_bindings_functions: [Function_descriptor_type; 1] = [Function_descriptor_type {
    Name: "Xila_graphics_call",
    Pointer: Call as *mut _,
}];
