use miniserde::{Deserialize, Serialize};
use File_system::{Mode_type, Path_type};
use Virtual_file_system::File_type;

use crate::Error::{Error_type, Result_type};

pub const Shortcut_path: &Path_type =
    Path_type::From_str("/Configuration/Graphical_shell/Shortcuts");

#[derive(Debug, Clone, Deserialize)]
pub struct Shortcut_type {
    Name: String,
    Command: String,
    Terminal: bool,
    Arguments: String,
    Icon_string: String,
}

impl Shortcut_type {
    pub fn Read(Name: &str, Buffer: &mut Vec<u8>) -> Result_type<Shortcut_type> {
        let Virtual_file_system = Virtual_file_system::Get_instance();

        let Shortcut_file_path = Shortcut_path
            .Append(Name)
            .ok_or(Error_type::Failed_to_get_shortcut_file_path)?;

        let Shortcut_file = File_type::Open(
            Virtual_file_system,
            Shortcut_file_path,
            Mode_type::Read_only.into(),
        )
        .map_err(Error_type::Failed_to_read_shortcut_file)?;

        Buffer.clear();

        Shortcut_file
            .Read_to_end(Buffer)
            .map_err(Error_type::Failed_to_read_shortcut_file)?;

        let String = core::str::from_utf8(Buffer).map_err(Error_type::Invalid_UTF_8)?;

        let Shortcut = Shortcut_type::From_str(String)?;

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
        self.Terminal
    }
}
