use Binding_tool::Bind_function_native;
use File_system::{
    Error_type, Flags_type, Path_type, Position_type, Result_type, Size_type,
    Unique_file_identifier_type, Virtual_file_system_type,
};
use Virtual_machine::{Function_descriptor_type, Function_descriptors, Registrable_trait};

pub struct File_system_bindings;

impl File_system_bindings {
    pub fn New() -> Self {
        Self {}
    }
}

impl Registrable_trait for File_system_bindings {
    fn Get_functions(&self) -> &[Function_descriptor_type] {
        &File_system_bindings_functions
    }
}

const File_system_bindings_functions: [Function_descriptor_type; 8] = Function_descriptors!(
    Open_binding,
    Close_file_binding,
    Read_binding,
    Write_binding,
    Create_file_binding,
    Exists_binding,
    Set_position_binding,
    Delete_binding
);

fn New_path(Path: &str) -> Result_type<&Path_type> {
    Path_type::New(Path).ok_or(Error_type::Invalid_path)
}

fn Get_virtual_file_system() -> &'static Virtual_file_system_type {
    File_system::Get_instance().expect("File system not initialized")
}

#[Bind_function_native(Prefix = "File_system")]
fn Open(
    Path: &str,
    Flags: Flags_type,
    File_identifier: &mut Unique_file_identifier_type,
) -> Result_type<()> {
    let Path = New_path(Path)?;

    *File_identifier = Get_virtual_file_system().Open(Path, Flags)?;

    Ok(())
}

#[Bind_function_native(Prefix = "File_system")]
fn Close_file(File_identifier: Unique_file_identifier_type) -> Result_type<()> {
    Get_virtual_file_system().Close(File_identifier)
}

#[Bind_function_native(Prefix = "File_system")]
fn Read(
    File_identifier: Unique_file_identifier_type,
    Buffer: &mut [u8],
    Read_size: &mut Size_type,
) -> Result_type<()> {
    *Read_size = Get_virtual_file_system().Read(File_identifier, Buffer)?;

    Ok(())
}

#[Bind_function_native(Prefix = "File_system")]
fn Write(
    File_identifier: Unique_file_identifier_type,
    Buffer: &[u8],
    Write_size: &mut Size_type,
) -> Result_type<()> {
    *Write_size = Get_virtual_file_system().Write(File_identifier, Buffer)?;

    Ok(())
}

#[Bind_function_native(Prefix = "File_system")]
fn Flush(File_identifier: Unique_file_identifier_type) -> Result_type<()> {
    Get_virtual_file_system().Flush(File_identifier)
}

#[Bind_function_native(Prefix = "File_system")]
fn Get_file_type(Path: &str, Type: &mut u32) -> Result_type<()> {
    let Path = New_path(Path)?;

    *Type = Get_virtual_file_system().Get_type(Path)? as u32;

    Ok(())
}

#[Bind_function_native(Prefix = "File_system")]
fn Get_file_size(Path: &str, Size: &mut Size_type) -> Result_type<()> {
    let Path = New_path(Path)?;

    *Size = Get_virtual_file_system().Get_size(Path)?;

    Ok(())
}

#[Bind_function_native(Prefix = "File_system")]
fn Set_position(
    File_identifier: Unique_file_identifier_type,
    Position: &Position_type,
    Result_value: &mut Size_type,
) -> Result_type<()> {
    *Result_value = Get_virtual_file_system().Set_position(File_identifier, Position)?;

    Ok(())
}

#[Bind_function_native(Prefix = "File_system")]
fn Delete_file(Path: &str, Recursive: bool) -> Result_type<()> {
    let Path = New_path(Path)?;

    Get_virtual_file_system().Delete(Path, Recursive)?;

    Ok(())
}

#[Bind_function_native(Prefix = "File_system")]
fn Create_file(Path: &str) -> Result_type<()> {
    let Path = New_path(Path)?;

    Get_virtual_file_system()
        .Create_file(Path)
        .expect("Failed to create file");

    Ok(())
}

#[Bind_function_native(Prefix = "File_system")]
fn Create_directory(Path: &str, Recursive: bool) -> Result_type<()> {
    let Path = New_path(Path)?;

    Get_virtual_file_system().Create_directory(Path, Recursive)?;

    Ok(())
}

#[Bind_function_native(Prefix = "File_system")]
fn Delete(Path: &str, Recursive: bool) -> Result_type<()> {
    let Path = New_path(Path)?;

    Get_virtual_file_system().Delete(Path, Recursive)?;

    Ok(())
}

#[Bind_function_native(Prefix = "File_system")]
fn Exists(Path: &str, Exists: &mut bool) -> Result_type<()> {
    let Path = New_path(Path)?;

    *Exists = Get_virtual_file_system().Exists(Path)?;

    Ok(())
}
