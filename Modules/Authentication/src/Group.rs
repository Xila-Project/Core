use miniserde::{Deserialize, Serialize};
use File_system::{Flags_type, Mode_type, Open_type, Path_owned_type, Path_type};
use Users::{
    Group_identifier_inner_type, Group_identifier_type, User_identifier_inner_type,
    User_identifier_type,
};
use Virtual_file_system::{Directory_type, File_type, Virtual_file_system_type};

use crate::{Error_type, Group_folder_path, Result_type};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Group_type {
    Identifier: Group_identifier_inner_type,
    Name: String,
    Users: Vec<User_identifier_inner_type>,
}

impl Group_type {
    pub fn New(
        Identifier: Group_identifier_inner_type,
        Name: String,
        Users: Vec<User_identifier_inner_type>,
    ) -> Self {
        Self {
            Identifier,
            Name,
            Users,
        }
    }

    pub fn Get_identifier(&self) -> Group_identifier_type {
        Group_identifier_type::New(self.Identifier)
    }

    pub fn Get_name(&self) -> &str {
        &self.Name
    }

    pub fn Get_users(&self) -> &[User_identifier_type] {
        // Avoid to copy the vector since User_identifier_type is transparent to User_identifier_inner_type.
        unsafe { core::mem::transmute(self.Users.as_slice()) }
    }
}

pub fn Get_group_file_path(Group_name: &str) -> Result_type<Path_owned_type> {
    Path_type::New(Group_folder_path)
        .to_owned()
        .Append(Group_name)
        .ok_or(Error_type::Failed_to_get_group_file_path)
}

pub fn Read_group_file<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Buffer: &mut Vec<u8>,
    File: &str,
) -> Result_type<Group_type> {
    let Group_file_path = Path_type::New(Group_folder_path)
        .to_owned()
        .Append(File)
        .ok_or(Error_type::Failed_to_get_group_file_path)?;

    let Group_file = File_type::Open(
        Virtual_file_system,
        Group_file_path,
        Mode_type::Read_only.into(),
    )
    .map_err(Error_type::Failed_to_read_group_directory)?;

    Buffer.clear();

    Group_file
        .Read_to_end(Buffer)
        .map_err(Error_type::Failed_to_read_group_file)?;

    miniserde::json::from_str(core::str::from_utf8(Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_group_file)
}

pub fn Create_group<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Group_name: &str,
    Group_identifier: Option<Group_identifier_type>,
) -> Result_type<Group_identifier_type> {
    let Users_manager = Users::Get_instance();

    // - New group identifier if not provided.
    let Group_identifier = if let Some(Group_identifier) = Group_identifier {
        Group_identifier
    } else {
        Users_manager
            .Get_new_group_identifier()
            .map_err(Error_type::Failed_to_get_new_group_identifier)?
    };

    // - Add it to the users manager.
    Users_manager
        .Add_group(Group_identifier, Group_name, &[])
        .map_err(Error_type::Failed_to_add_group)?;

    // - Write group file.
    let Group = Group_type::New(
        Group_identifier.Into_inner(),
        Group_name.to_string(),
        vec![],
    );

    match Directory_type::Create(Virtual_file_system, Group_folder_path) {
        Ok(_) | Err(File_system::Error_type::Already_exists) => {}
        Err(Error) => Err(Error_type::Failed_to_create_groups_directory(Error))?,
    };

    let Group_file_path = Get_group_file_path(Group_name)?;

    let Group_file = File_type::Open(
        Virtual_file_system,
        Group_file_path,
        Flags_type::New(Mode_type::Write_only, Some(Open_type::Create_only), None),
    )
    .map_err(Error_type::Failed_to_open_group_file)?;

    let Group_json = miniserde::json::to_string(&Group);

    Group_file
        .Write(Group_json.as_bytes())
        .map_err(Error_type::Failed_to_write_group_file)?;

    Ok(Group_identifier)
}
