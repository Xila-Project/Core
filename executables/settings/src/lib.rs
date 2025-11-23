#![no_std]

extern crate alloc;

mod error;
mod settings;
mod tabs;

use alloc::boxed::Box;
pub use error::*;
pub use settings::*;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::num::NonZeroUsize;
use xila::executable::{self, ExecutableTrait, Standard};
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
            .create_directory(task, &"/configuration/shared/shortcuts")
            .await;

        File::write_to_path(
            virtual_file_system,
            task,
            "/configuration/shared/shortcuts/settings.json",
            get_shortcut().as_bytes(),
        )
        .await
        .map_err(|error| error.to_string())?;

        Ok(Self)
    }
}

impl ExecutableTrait for SettingsExecutable {
    fn main(standard: Standard, arguments: Vec<String>) -> executable::MainFuture {
        Box::pin(async move { main(standard, arguments).await })
    }
}

pub async fn main(_: Standard, _: Vec<String>) -> core::result::Result<(), NonZeroUsize> {
    let mut settings = Settings::new()
        .await
        .map_err(|_| NonZeroUsize::new(1).unwrap())?;

    // Run the main loop
    settings.run().await;

    Ok(())
}
