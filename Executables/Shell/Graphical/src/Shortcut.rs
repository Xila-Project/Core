use alloc::{string::String, vec::Vec};
use miniserde::{Deserialize, Serialize};
use File_system::{Mode_type, Path_type};
use Graphics::Color_type;
use Virtual_file_system::File_type;

use crate::Error::{Error_type, Result_type};

pub const SHORTCUT_PATH: &Path_type = Path_type::From_str("/Configuration/Shared/Shortcuts");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Shortcut_type {
    Name: String,
    Command: String,
    #[serde(rename = "Terminal")]
    _Terminal: bool,
    Arguments: String,
    Icon_string: String,
    Icon_color: [u8; 3],
}

impl Shortcut_type {
    pub async fn Add(Path: &Path_type) -> Result_type<()> {
        let Shortcut = Shortcut_type::Read_from_path(Path, &mut Vec::new()).await?;

        let New_shortcut_path = SHORTCUT_PATH
            .Append(Shortcut.Get_name())
            .ok_or(Error_type::Failed_to_get_shortcut_file_path)?
            .Append(".json")
            .ok_or(Error_type::Failed_to_get_shortcut_file_path)?;

        Virtual_file_system::Get_instance()
            .Rename(&Path, &New_shortcut_path)
            .await
            .map_err(Error_type::Failed_to_add_shortcut)?;

        Ok(())
    }

    pub async fn Read_from_path(
        Path: &Path_type,
        Buffer: &mut Vec<u8>,
    ) -> Result_type<Shortcut_type> {
        let Virtual_file_system = Virtual_file_system::Get_instance();

        let Shortcut_file = File_type::Open(Virtual_file_system, Path, Mode_type::READ_ONLY.into())
            .await
            .map_err(Error_type::Failed_to_read_shortcut_file)?;

        Buffer.clear();

        Shortcut_file
            .Read_to_end(Buffer)
            .await
            .map_err(Error_type::Failed_to_read_shortcut_file)?;

        let String = core::str::from_utf8(Buffer).map_err(Error_type::Invalid_UTF_8)?;

        let Shortcut = Shortcut_type::From_str(String)?;

        Ok(Shortcut)
    }

    pub async fn Read(Entry_name: &str, Buffer: &mut Vec<u8>) -> Result_type<Shortcut_type> {
        let Shortcut_file_path = SHORTCUT_PATH
            .Append(Entry_name)
            .ok_or(Error_type::Failed_to_get_shortcut_file_path)?;

        let Shortcut = Shortcut_type::Read_from_path(&Shortcut_file_path, Buffer).await?;

        Ok(Shortcut)
    }

    pub fn From_str(String: &str) -> Result_type<Self> {
        miniserde::json::from_str(String).map_err(Error_type::Failed_to_deserialize_shortcut)
    }

    pub fn Get_name(&self) -> &str {
        &self.Name
    }

    pub fn Get_icon_string(&self) -> &str {
        &self.Icon_string
    }

    pub fn Get_command(&self) -> &str {
        &self.Command
    }

    pub fn Get_arguments(&self) -> &str {
        &self.Arguments
    }

    pub fn Get_icon_color(&self) -> Color_type {
        Color_type::New(self.Icon_color[0], self.Icon_color[1], self.Icon_color[2])
    }

    // pub fn Is_terminal(&self) -> bool {
    //     self._Terminal
    // }
}
