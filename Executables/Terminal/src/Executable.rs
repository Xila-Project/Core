use alloc::string::{String, ToString};
use Executable::Implement_executable_device;
use File_system::{Flags_type, Mode_type, Open_type};
use Task::Task_identifier_type;
use Virtual_file_system::{File_type, Virtual_file_system_type};

use crate::Main::Main;

pub struct Terminal_executable_type;

impl Terminal_executable_type {
    pub async fn New<'a>(
        Virtual_file_system: &'a Virtual_file_system_type<'a>,
        Task: Task_identifier_type,
    ) -> Result<Self, String> {
        let _ = Virtual_file_system
            .Create_directory(&"/Configuration/Shared/Shortcuts", Task)
            .await;

        let File = match File_type::Open(
            Virtual_file_system,
            "/Configuration/Shared/Shortcuts/Terminal.json",
            Flags_type::New(Mode_type::WRITE_ONLY, Open_type::CREATE_ONLY.into(), None),
        )
        .await
        {
            Ok(File) => File,
            Err(File_system::Error_type::Already_exists) => {
                return Ok(Self);
            }
            Err(Error) => Err(Error.to_string())?,
        };

        File.Write(crate::SHORTCUT.as_bytes())
            .await
            .map_err(|Error| Error.to_string())?;

        Ok(Self)
    }
}

Implement_executable_device!(
    Structure: Terminal_executable_type,
    Mount_path: "/Binaries/Terminal",
    Main_function: Main,
);
