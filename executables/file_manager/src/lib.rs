#![no_std]

extern crate alloc;

mod error;
mod file_manager;

use alloc::string::{String, ToString};
use core::num::NonZeroUsize;
pub use error::*;
pub use file_manager::*;
use file_system::{Flags, Mode, Open};
use task::TaskIdentifier;
use virtual_file_system::{File, VirtualFileSystemType};

use executable::Standard;

pub const SHORTCUT: &str = r#"
{
    "name": "File manager",
    "command": "/Binaries/File_manager",
    "arguments": "",
    "terminal": false,
    "icon_string": "Fm",
    "icon_color": [0, 188, 212]
}"#;

pub struct FileManagerExecutableType;

impl FileManagerExecutableType {
    pub async fn new<'a>(
        virtual_file_system: &'a VirtualFileSystemType<'a>,
        task: TaskIdentifier,
    ) -> core::result::Result<Self, String> {
        let _ = virtual_file_system
            .create_directory(&"/Configuration/Shared/Shortcuts", task)
            .await;

        let file = match File::open(
            virtual_file_system,
            "/Configuration/Shared/Shortcuts/File_manager.json",
            Flags::new(Mode::WRITE_ONLY, Open::CREATE_ONLY.into(), None),
        )
        .await
        {
            Ok(file) => file,
            Err(file_system::Error::AlreadyExists) => {
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

executable::implement_executable_device!(
    Structure: FileManagerExecutableType,
    Mount_path: "/Binaries/File_manager",
    Main_function: main,
);

pub async fn main(_: Standard, _: String) -> core::result::Result<(), NonZeroUsize> {
    let mut file_manager = FileManagerType::new()
        .await
        .map_err(|_| NonZeroUsize::new(1).unwrap())?;

    // Run the main loop
    file_manager.run().await;

    Ok(())
}
