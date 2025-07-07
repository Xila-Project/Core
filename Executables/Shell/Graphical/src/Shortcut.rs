use alloc::{string::String, vec::Vec};
use miniserde::{Deserialize, Serialize};
use File_system::{Mode_type, Path_type};
use Graphics::Color_type;
use Virtual_file_system::File_type;

use crate::Error::{Error_type, Result_type};

pub const SHORTCUT_PATH: &Path_type = Path_type::From_str("/Configuration/Shared/Shortcuts");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Shortcut_type {
    name: String,
    command: String,
    #[serde(rename = "Terminal")]
    _terminal: bool,
    arguments: String,
    icon_string: String,
    icon_color: [u8; 3],
}

impl Shortcut_type {
    pub async fn add(path: &Path_type) -> Result_type<()> {
        let shortcut = Shortcut_type::Read_from_path(path, &mut Vec::new()).await?;

        let New_shortcut_path = SHORTCUT_PATH
            .Append(shortcut.Get_name())
            .ok_or(Error_type::Failed_to_get_shortcut_file_path)?
            .Append(".json")
            .ok_or(Error_type::Failed_to_get_shortcut_file_path)?;

        Virtual_file_system::Get_instance()
            .Rename(&path, &New_shortcut_path)
            .await
            .map_err(Error_type::Failed_to_add_shortcut)?;

        Ok(())
    }

    pub async fn Read_from_path(
        path: &Path_type,
        buffer: &mut Vec<u8>,
    ) -> Result_type<Shortcut_type> {
        let virtual_file_system = Virtual_file_system::Get_instance();

        let Shortcut_file = File_type::Open(virtual_file_system, path, Mode_type::READ_ONLY.into())
            .await
            .map_err(Error_type::Failed_to_read_shortcut_file)?;

        buffer.clear();

        Shortcut_file
            .Read_to_end(buffer)
            .await
            .map_err(Error_type::Failed_to_read_shortcut_file)?;

        let String = core::str::from_utf8(buffer).map_err(Error_type::Invalid_UTF_8)?;

        let Shortcut = Shortcut_type::From_str(String)?;

        Ok(Shortcut)
    }

    pub async fn Read(Entry_name: &str, Buffer: &mut Vec<u8>) -> Result_type<Shortcut_type> {
        let shortcut_file_path = SHORTCUT_PATH
            .Append(Entry_name)
            .ok_or(Error_type::Failed_to_get_shortcut_file_path)?;

        let Shortcut = Shortcut_type::Read_from_path(&shortcut_file_path, Buffer).await?;

        Ok(Shortcut)
    }

    pub fn From_str(String: &str) -> Result_type<Self> {
        miniserde::json::from_str(String).map_err(Error_type::Failed_to_deserialize_shortcut)
    }

    pub fn Get_name(&self) -> &str {
        &self.name
    }

    pub fn Get_icon_string(&self) -> &str {
        &self.icon_string
    }

    pub fn Get_command(&self) -> &str {
        &self.command
    }

    pub fn Get_arguments(&self) -> &str {
        &self.arguments
    }

    pub fn Get_icon_color(&self) -> Color_type {
        Color_type::New(self.icon_color[0], self.icon_color[1], self.icon_color[2])
    }

    // pub fn Is_terminal(&self) -> bool {
    //     self._Terminal
    // }
}
