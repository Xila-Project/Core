use std::num::NonZeroU32;

use Binding_tool::Bind_function_WASM;

#[Bind_function_WASM]
fn Open(Path: &str, Flags: u32, File_identifier: &mut u32) -> Result<(), NonZeroU32> {}

#[Bind_function_WASM]
fn Read(File_identifier: u32, Buffer: &mut [u8], Read_size: &mut u64) -> Result<(), NonZeroU32> {}

#[Bind_function_WASM]
fn Create_file(Path: &str) -> Result<(), NonZeroU32> {}

#[Bind_function_WASM]
fn Write(File_identifier: u32, Buffer: &[u8], Write_size: &mut u64) -> Result<(), NonZeroU32> {}

#[Bind_function_WASM]
fn Exists(Path: &str, Exists: &mut bool) -> Result<(), NonZeroU32> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
#[allow(dead_code)]
pub enum Position_type {
    Start(u64),
    Current(i64),
    End(i64),
}

#[Bind_function_WASM]
fn Set_position(
    File_identifier: u32,
    Position: &Position_type,
    Result_value: &mut u64,
) -> Result<(), NonZeroU32> {
}

#[Bind_function_WASM]
fn Delete(Path: &str, Recursive: bool) -> Result<(), NonZeroU32> {}

#[no_mangle]
fn Test_file_system() -> u32 {
    let mut File_identifier = 0;

    let mut File_exists: bool = true;

    Exists("/wasm.txt", &mut File_exists).expect("Failed to check if file exists");

    if File_exists {
        Delete("/wasm.txt", false).expect("Failed to delete file");
    }

    Create_file("/wasm.txt").expect("Failed to create file");

    Exists("/wasm.txt", &mut File_exists).expect("Failed to check if file exists");

    if !File_exists {
        return 2;
    }

    let Flags = 0b11 << 4;

    Open("/wasm.txt", Flags, &mut File_identifier).expect("Failed to open file");

    let mut Write_size = 0;

    let Message = "Hello world from WASM !";

    Write(File_identifier, Message.as_bytes(), &mut Write_size).expect("Failed to write file");

    if Write_size != Message.len() as u64 {
        return 4;
    }

    let mut Result_position = 0;

    Set_position(
        File_identifier,
        &Position_type::Start(0),
        &mut Result_position,
    )
    .expect("Failed to set position");

    let mut Buffer = [0; 64];

    let mut Read_size = 0;

    Read(File_identifier, &mut Buffer, &mut Read_size).expect("Failed to read file");

    let String = String::from_utf8_lossy(&Buffer[..Read_size as usize]);

    if String != Message {
        return 6;
    }

    Delete("/wasm.txt", false).expect("Failed to delete file");

    0
}
