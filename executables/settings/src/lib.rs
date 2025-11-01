#![no_std]

extern crate alloc;

mod error;
mod settings;
mod tabs;

pub use error::*;
pub use settings::*;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::num::NonZeroUsize;
use xila::executable::{self, Standard};
use xila::file_system::{self, Flags, Mode, Open};
use xila::task::TaskIdentifier;
use xila::virtual_file_system::{File, VirtualFileSystem};

pub fn get_shortcut() -> alloc::string::String {
    use xila::internationalization::translate;
    alloc::format!(
        r#"{{
    "name": "{}",
    "command": "/binaries/settings",
    "arguments": [],
    "terminal": false,
    "icon_string": "Se",
    "icon_color": [158, 158, 158]
}}"#,
        translate!("Settings")
    )
}

pub struct SettingsExecutable;

impl SettingsExecutable {
    pub async fn new<'a>(
        virtual_file_system: &'a VirtualFileSystem<'a>,
        task: TaskIdentifier,
    ) -> core::result::Result<Self, String> {
        let _ = virtual_file_system
            .create_directory(&"/configuration/shared/shortcuts", task)
            .await;

        let file = match File::open(
            virtual_file_system,
            "/configuration/shared/shortcuts/settings.json",
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

        let shortcut = get_shortcut();
        file.write(shortcut.as_bytes())
            .await
            .map_err(|error| error.to_string())?;

        Ok(Self)
    }
}

executable::implement_executable_device!(
    structure: SettingsExecutable,
    mount_path: "/binaries/settings",
    main_function: main,
);

pub async fn main(_: Standard, _: Vec<String>) -> core::result::Result<(), NonZeroUsize> {
    let mut settings = Settings::new()
        .await
        .map_err(|_| NonZeroUsize::new(1).unwrap())?;

    // Run the main loop
    settings.run().await;

    Ok(())
}
