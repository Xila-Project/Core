use alloc::{string::String, vec::Vec};
use file_system::{Mode, Path};
use graphics::Color;
use miniserde::{Deserialize, Serialize};
use virtual_file_system::File;

use crate::error::{Error, Result};

pub const SHORTCUT_PATH: &Path = Path::from_str("/configuration/shared/shortcuts");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Shortcut {
    name: String,
    command: String,
    #[serde(rename = "terminal")]
    _terminal: bool,
    arguments: String,
    icon_string: String,
    icon_color: [u8; 3],
}

impl Shortcut {
    pub async fn add(path: &Path) -> Result<()> {
        let shortcut = Shortcut::read_from_path(path, &mut Vec::new()).await?;

        let new_shortcut_path = SHORTCUT_PATH
            .append(shortcut.get_name())
            .ok_or(Error::FailedToGetShortcutFilePath)?
            .append(".json")
            .ok_or(Error::FailedToGetShortcutFilePath)?;

        virtual_file_system::get_instance()
            .rename(&path, &new_shortcut_path)
            .await
            .map_err(Error::FailedToAddShortcut)?;

        Ok(())
    }

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

        log::information!("Shortcut buffer: {buffer:?}");

        let string = core::str::from_utf8(buffer).map_err(Error::InvalidUtf8)?;

        log::information!("Reading shortcut from path: {path:?}");
        log::information!("Shortcut content: {string}");

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
        miniserde::json::from_str(string).map_err(|e| {
            log::error!("Failed to deserialize shortcut: {e}");
            log::error!("String: {string}");

            Error::FailedToDeserializeShortcut(e)
        })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_icon_string(&self) -> &str {
        &self.icon_string
    }

    pub fn get_command(&self) -> &str {
        &self.command
    }

    pub fn get_arguments(&self) -> &str {
        &self.arguments
    }

    pub fn get_icon_color(&self) -> Color {
        Color::new(self.icon_color[0], self.icon_color[1], self.icon_color[2])
    }

    // pub fn is_terminal(&self) -> bool {
    //     self._Terminal
    // }
}
