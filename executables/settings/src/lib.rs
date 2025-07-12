#![no_std]

extern crate alloc;

mod error;
mod settings;
mod tabs;
use core::num::NonZeroUsize;

use executable::Standard;

use alloc::string::{String, ToString};
pub use error::*;
use file_system::{Flags, Mode, Open};
pub use settings::*;
use task::TaskIdentifier;
use virtual_file_system::{File, VirtualFileSystemType};

pub const SHORTCUT: &str = r#"
{
    "name": "Settings",
    "command": "/Binaries/Settings",
    "arguments": "",
    "terminal": false,
    "icon_string": "Se",
    "icon_color": [158, 158, 158]
}"#;

pub struct SettingsExecutableType;

impl SettingsExecutableType {
    pub async fn new<'a>(
        virtual_file_system: &'a VirtualFileSystemType<'a>,
        task: TaskIdentifier,
    ) -> core::result::Result<Self, String> {
        let _ = virtual_file_system
            .create_directory(&"/Configuration/Shared/Shortcuts", task)
            .await;

        let file = match File::open(
            virtual_file_system,
            "/Configuration/Shared/Shortcuts/Settings.json",
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
    Structure: SettingsExecutableType,
    Mount_path: "/Binaries/Settings",
    Main_function: main,
);

pub async fn main(_: Standard, _: String) -> core::result::Result<(), NonZeroUsize> {
    let mut settings = SettingsType::new()
        .await
        .map_err(|_| NonZeroUsize::new(1).unwrap())?;

    // Run the main loop
    settings.run().await;

    Ok(())
}
