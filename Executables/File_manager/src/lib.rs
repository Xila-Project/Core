#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

mod error;
mod file_manager;

use alloc::string::{String, ToString};
use core::num::NonZeroUsize;
pub use error::*;
pub use file_manager::*;
use file_system::{Flags_type, Mode_type, Open_type};
use task::Task_identifier_type;
use virtual_file_system::{File_type, Virtual_file_system_type};

use executable::Standard_type;

use crate::File_manager_type;

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
            .create_directory(&"/Configuration/Shared/Shortcuts", task)
            .await;

        let file = match File_type::open(
            virtual_file_system,
            "/Configuration/Shared/Shortcuts/File_manager.json",
            Flags_type::New(Mode_type::WRITE_ONLY, Open_type::CREATE_ONLY.into(), None),
        )
        .await
        {
            Ok(file) => file,
            Err(File_system::Error_type::Already_exists) => {
                return Ok(Self);
            }
            Err(error) => Err(error.to_string())?,
        };

        file.write(crate::SHORTCUT.as_bytes())
            .await
            .map_err(|error| error.to_string())?;

        Ok(Self)
    }
}

Executable::Implement_executable_device!(
    Structure: File_manager_executable_type,
    Mount_path: "/Binaries/File_manager",
    Main_function: main,
);

pub async fn main(_: Standard_type, _: String) -> Result<(), NonZeroUsize> {
    let mut file_manager = File_manager_type::new()
        .await
        .map_err(|_| NonZeroUsize::new(1).unwrap())?;

    // Run the main loop
    file_manager.run().await;

    Ok(())
}
