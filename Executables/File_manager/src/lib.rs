#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

mod Error;
mod File_manager;
mod Main;

use alloc::string::{String, ToString};
pub use Error::*;
pub use File_manager::*;
use File_system::{Flags_type, Mode_type, Open_type};
use Task::Task_identifier_type;
use Virtual_file_system::{File_type, Virtual_file_system_type};

pub const SHORTCUT: &str = r#"
{
    "Name": "File manager",
    "Command": "/Binaries/File_manager",
    "Arguments": "",
    "Terminal": false,
    "Icon_string": "Fm",
    "Icon_color": [0, 188, 212]
}"#;

pub struct File_manager_executable_type;

impl File_manager_executable_type {
    pub async fn new<'a>(
        virtual_file_system: &'a Virtual_file_system_type<'a>,
        task: Task_identifier_type,
    ) -> Result<Self, String> {
        let _ = virtual_file_system
            .Create_directory(&"/Configuration/Shared/Shortcuts", task)
            .await;

        let File = match File_type::Open(
            virtual_file_system,
            "/Configuration/Shared/Shortcuts/File_manager.json",
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

Executable::Implement_executable_device!(
    Structure: File_manager_executable_type,
    Mount_path: "/Binaries/File_manager",
    Main_function: Main::Main,
);
