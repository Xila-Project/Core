use Executable::Read_data_type;
use File_system::{Device_trait, Flags_type, Mode_type, Open_type};
use Task::Task_identifier_type;
use Virtual_file_system::{File_type, Virtual_file_system_type};

use crate::Main::Main;

pub struct Terminal_executable_type;

impl Terminal_executable_type {
    pub fn New<'a>(
        Virtual_file_system: &'a Virtual_file_system_type<'a>,
        Task: Task_identifier_type,
    ) -> Result<Self, String> {
        let _ = Virtual_file_system.Create_directory(&"/Configuration/Shared/Shortcuts", Task);

        let File = match File_type::Open(
            Virtual_file_system,
            "/Configuration/Shared/Shortcuts/Terminal.json",
            Flags_type::New(Mode_type::Write_only, Open_type::Create_only.into(), None),
        ) {
            Ok(File) => File,
            Err(File_system::Error_type::Already_exists) => {
                return Ok(Self);
            }
            Err(Error) => Err(Error.to_string())?,
        };

        File.Write(crate::Shortcut.as_bytes())
            .map_err(|Error| Error.to_string())?;

        Ok(Self)
    }
}

impl Device_trait for Terminal_executable_type {
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
