use miniserde::Deserialize;
use File_system::{Mode_type, Path_type};
use Virtual_file_system::File_type;

use crate::Error::{Error_type, Result_type};

pub const Shortcut_path: &Path_type = Path_type::From_str("/Configuration/Shared/Shortcuts");

#[derive(Debug, Clone, Deserialize)]
pub struct Shortcut_type {
    Name: String,
    Command: String,
    #[serde(rename = "Terminal")]
    _Terminal: bool,
    Arguments: String,
    Icon_string: String,
}

impl Shortcut_type {
    pub fn Add(Path: &Path_type) -> Result_type<()> {
        let Shortcut = Shortcut_type::Read_from_path(Path, &mut Vec::new())?;

        let New_shortcut_path = Shortcut_path
            .Append(Shortcut.Get_name())
            .ok_or(Error_type::Failed_to_get_shortcut_file_path)?
            .Append(".json")
            .ok_or(Error_type::Failed_to_get_shortcut_file_path)?;

        Virtual_file_system::Get_instance()
            .Rename(&Path, &New_shortcut_path)
            .map_err(Error_type::Failed_to_add_shortcut)?;

        Ok(())
    }

    pub fn Read_from_path(Path: &Path_type, Buffer: &mut Vec<u8>) -> Result_type<Shortcut_type> {
        let Virtual_file_system = Virtual_file_system::Get_instance();

        let Shortcut_file = File_type::Open(Virtual_file_system, Path, Mode_type::Read_only.into())
            .map_err(Error_type::Failed_to_read_shortcut_file)?;

        Buffer.clear();

        Shortcut_file
            .Read_to_end(Buffer)
            .map_err(Error_type::Failed_to_read_shortcut_file)?;

        let String = core::str::from_utf8(Buffer).map_err(Error_type::Invalid_UTF_8)?;

        let Shortcut = Shortcut_type::From_str(String)?;

        Ok(Shortcut)
    }

    pub fn Read(Entry_name: &str, Buffer: &mut Vec<u8>) -> Result_type<Shortcut_type> {
        let Shortcut_file_path = Shortcut_path
            .Append(Entry_name)
            .ok_or(Error_type::Failed_to_get_shortcut_file_path)?;

        let Shortcut = Shortcut_type::Read_from_path(&Shortcut_file_path, Buffer)?;

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

    pub fn Is_terminal(&self) -> bool {
        self._Terminal
    }
}
