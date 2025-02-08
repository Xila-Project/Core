use Executable::{Device_executable_trait, Read_data_type};
use File_system::{Create_device, Device_trait};
use Task::Task_identifier_type;
use Virtual_file_system::Virtual_file_system_type;

use crate::Main::Main;

pub struct Shell_executable_type;

impl Device_executable_trait for Shell_executable_type {
    fn Mount<'a>(
        Virtual_file_system: &'a Virtual_file_system_type<'a>,
        Task: Task_identifier_type,
    ) -> Result<(), String> {
        Virtual_file_system
            .Mount_static_device(
                Task,
                &"/Binaries/Command_line_shell",
                Create_device!(Shell_executable_type),
            )
            .map_err(|Error| Error.to_string())?;

        Ok(())
    }
}

impl Device_trait for Shell_executable_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<File_system::Size_type> {
        let Read_data: &mut Read_data_type = Buffer
            .try_into()
            .map_err(|_| File_system::Error_type::Invalid_parameter)?;

        *Read_data = Read_data_type::New(Main, 1024 * 32);

        Ok(size_of::<Read_data_type>().into())
    }

    fn Write(&self, _: &[u8]) -> File_system::Result_type<File_system::Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Get_size(&self) -> File_system::Result_type<File_system::Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Set_position(
        &self,
        _: &File_system::Position_type,
    ) -> File_system::Result_type<File_system::Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Err(File_system::Error_type::Unsupported_operation)
    }
}
