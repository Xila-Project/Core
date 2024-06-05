#![allow(non_snake_case)]

use Binding_tool::Bind_function_WASM;

#[Bind_function_WASM]
fn Open_file(Path: &str, Mode: u32, File_identifier: &mut u16) -> u32 {}

#[Bind_function_WASM]
fn Read_file(File_identifier: u16, Buffer: &mut [u8], Read_size: &mut u32) -> u32 {}

extern crate wee_alloc;

// Use `wee_alloc` as the global allocator saving kilobytes of code size
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() -> Result<(), ()> {
    let mut File_identifier = 1;

    let Result = Open_file("test.txt", 1, &mut File_identifier);

    if Result != 0 {
        return Err(());
    }

    let mut Buffer = [0; 1024];

    let mut Read_size = 0;

    let Result = Read_file(File_identifier, &mut Buffer, &mut Read_size);

    if Result != 0 {
        return Err(());
    }

    let String = String::from_utf8_lossy(&Buffer[..Read_size as usize]);

    if String != "Hello World!" {
        return Err(());
    }

    Ok(())
}
