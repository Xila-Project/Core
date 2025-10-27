use alloc::{string::String, vec::Vec};
use miniserde::{Deserialize, Serialize};
use xila::file_system::{Mode, Path};
use xila::graphics::Color;
use xila::virtual_file_system::{self, File};

use crate::error::{Error, Result};

pub const SHORTCUT_PATH: &Path = Path::from_str("/configuration/shared/shortcuts");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Shortcut {
    pub name: String,
    pub command: String,
    #[serde(rename = "terminal")]
    pub _terminal: bool,
    pub arguments: Vec<String>,
    pub icon_string: String,
    icon_color: [u8; 3],
}

impl Shortcut {
    pub async fn read_from_path(path: &Path, buffer: &mut Vec<u8>) -> Result<Shortcut> {
        let virtual_file_system = virtual_file_system::get_instance();

        let shortcut_file = File::open(virtual_file_system, path, Mode::READ_ONLY.into())
            .await
            .map_err(Error::FailedToReadShortcutFile)?;

        buffer.clear();

        shortcut_file
            .read_to_end(buffer)
            .await
            .map_err(Error::FailedToReadShortcutFile)?;

        let string = core::str::from_utf8(buffer).map_err(Error::InvalidUtf8)?;

        let shortcut = Shortcut::from_str(string)?;

        Ok(shortcut)
    }

    pub async fn read(entry_name: &str, buffer: &mut Vec<u8>) -> Result<Shortcut> {
        let shortcut_file_path = SHORTCUT_PATH
            .append(entry_name)
            .ok_or(Error::FailedToGetShortcutFilePath)?;

        let shortcut = Shortcut::read_from_path(&shortcut_file_path, buffer).await?;

        Ok(shortcut)
    }

    pub fn from_str(string: &str) -> Result<Self> {
        miniserde::json::from_str(string).map_err(Error::FailedToDeserializeShortcut)
    }

    pub fn get_icon_color(&self) -> Color {
        Color::new(self.icon_color[0], self.icon_color[1], self.icon_color[2])
    }

    // pub fn is_terminal(&self) -> bool {
    //     self._Terminal
    // }
}
