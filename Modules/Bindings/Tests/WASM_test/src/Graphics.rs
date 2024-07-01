use std::thread::sleep;

use Binding_tool::Bind_function_WASM;

#[repr(transparent)]
pub struct Object_type([u8; 64]);

impl Default for Object_type {
    fn default() -> Self {
        Self([0; 64])
    }
}

#[Bind_function_WASM]
fn Create_calendar(Result: &mut Object_type) {}

#[Bind_function_WASM]
fn Delete_object(Object: &mut Object_type) {}

#[no_mangle]
fn Test_graphics() -> u32 {
    let mut Object = Object_type::default();

    Create_calendar(&mut Object);

    // sleep(std::time::Duration::from_secs(1));

    // Delete_object(&mut Object);

    42
}
