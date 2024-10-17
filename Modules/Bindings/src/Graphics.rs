pub use Graphics::lvgl;

use Virtual_machine::{
    Environment_pointer_type, Environment_type, Function_descriptor_type, Registrable_trait,
    WASM_pointer_type, WASM_usize_type,
};

include!(concat!(env!("OUT_DIR"), "/Bindings.rs"));

pub struct Graphics_bindings;

impl Registrable_trait for Graphics_bindings {
    fn Get_functions(&self) -> &[Function_descriptor_type] {
        &Graphics_bindings_functions
    }
}

impl Graphics_bindings {
    pub fn New() -> Self {
        Self {}
    }
}

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
        Generated_bindings::Call_function(
            Environment,
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
