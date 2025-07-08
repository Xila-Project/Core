use alloc::string::{String, ToString};
use executable::Implement_executable_device;
use file_system::{Flags_type, Mode_type, Open_type};
use task::Task_identifier_type;
use virtual_file_system::{File_type, Virtual_file_system_type};

use crate::Main::main;

pub struct Terminal_executable_type;

impl Terminal_executable_type {
    pub async fn new<'a>(
        virtual_file_system: &'a Virtual_file_system_type<'a>,
        task: Task_identifier_type,
    ) -> Result<Self, String> {
        let _ = virtual_file_system
            .create_directory(&"/Configuration/Shared/Shortcuts", task)
            .await;

        let File = match File_type::open(
            virtual_file_system,
            "/Configuration/Shared/Shortcuts/Terminal.json",
            Flags_type::New(Mode_type::WRITE_ONLY, Open_type::CREATE_ONLY.into(), None),
        )
        .await
        {
            Ok(File) => File,
            Err(File_system::Error_type::Already_exists) => {
                return Ok(Self);
            }
            Err(error) => Err(error.to_string())?,
        };

        File.write(crate::SHORTCUT.as_bytes())
            .await
            .map_err(|Error| Error.to_string())?;

        Ok(Self)
    }
}

Implement_executable_device!(
    Structure: Terminal_executable_type,
    Mount_path: "/Binaries/Terminal",
    Main_function: main,
);
