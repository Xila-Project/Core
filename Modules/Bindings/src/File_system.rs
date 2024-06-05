use Binding_tool::Bind_function_native;
use File_system::Prelude::*;
use Shared::{Discriminant_trait, From_result_to_u32};
use Virtual_machine::{Function_descriptor_type, Function_descriptors, Registrable_trait};
pub struct File_system_bindings {}

impl Registrable_trait for File_system_bindings {
    fn Get_functions(&self) -> &[Function_descriptor_type] {
        &File_system_bindings_functions
    }
}

const File_system_bindings_functions: [Function_descriptor_type; 4] = Function_descriptors!(
    Open_file_binding,
    Close_file_binding,
    Read_file_binding,
    Write_file_binding
);

#[Bind_function_native(Prefix = "File_system")]
fn Open_file(Path: &str, Mode: u32, File_identifier: &mut u16) -> u32 {
    println!("Open_file {:?}", Path);
    println!("Mode {:?}", Mode);
    println!("File_identifier {:?}", File_identifier);

    let File_system = Environment.Get_user_data().Get_file_system();

    let Path: Path_type = Path.into();

    let Result = File_system.Open_file(&Path, Mode.into());

    if let Ok(File) = &Result {
        *File_identifier = File.Get_identifier();
    } else if let Err(Error) = &Result {
        println!("Error {:?}", Error);
    }
    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Close_file(File_identifier: u16) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Result = File_system.Close_file(File_identifier);

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Read_file(File_identifier: u16, Buffer: &mut [u8], Read_size: &mut u32) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Result = File_system.Read_file(File_identifier, Buffer);

    if let Ok(Size) = &Result {
        *Read_size = *Size as u32;
    }

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Write_file(File_identifier: u16, Buffer: &[u8], Write_size: &mut u32) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Result = File_system.Write_file(File_identifier, Buffer);

    if let Ok(Size) = &Result {
        *Write_size = *Size as u32;
    }

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Flush_file(File_identifier: u16) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Result = File_system.Flush_file(File_identifier);

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Get_file_type(File_identifier: u16, Type_reference: &mut u32) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Result = File_system.Get_file_type(File_identifier);

    if let Ok(Type) = &Result {
        *Type_reference = Type.Get_discriminant();
    }

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Get_file_size(File_identifier: u16, Size_reference: &mut u64) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Result = File_system.Get_file_size(File_identifier);

    if let Ok(Size) = &Result {
        *Size_reference = Size.0;
    }

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Get_file_position(File_identifier: u16, Position_reference: &mut u64) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Result = File_system.Get_file_position(File_identifier);

    if let Ok(Position) = &Result {
        *Position_reference = Position.0;
    }

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Set_file_position(File_identifier: u16, Mode: u32, Value: u64) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Result = File_system.Set_file_position(File_identifier, &Position_type::From(Mode, Value));

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Delete_file(Path: &str) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Path: Path_type = Path.into();

    let Result = File_system.Delete_file(&Path);

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Create_directory(Path: &str) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Path: Path_type = Path.into();

    let Result = File_system.Create_directory(&Path);

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Create_directory_recursive(Path: &str) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Path: Path_type = Path.into();

    let Result = File_system.Create_directory_recursive(&Path);

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Delete_directory(Path: &str) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Path: Path_type = Path.into();

    let Result = File_system.Delete_directory(&Path);

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Delete_directory_recursive(Path: &str) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Path: Path_type = Path.into();

    let Result = File_system.Delete_directory_recursive(&Path);

    From_result_to_u32(&Result)
}

#[Bind_function_native(Prefix = "File_system")]
fn Move(Path: &str, Destination: &str) -> u32 {
    let File_system = Environment.Get_user_data().Get_file_system();

    let Path: Path_type = Path.into();
    let Destination: Path_type = Destination.into();

    let Result = File_system.Move(&Path, &Destination);

    From_result_to_u32(&Result)
}
