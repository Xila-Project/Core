use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};
use miniserde::{Deserialize, Serialize};
use File_system::{Flags_type, Mode_type, Open_type, Path_owned_type, Path_type};
use Users::{
    Group_identifier_inner_type, Group_identifier_type, User_identifier_inner_type,
    User_identifier_type,
};
use Virtual_file_system::{Directory_type, File_type, Virtual_file_system_type};

use crate::{
    Error_type,
    Hash::{Generate_salt, Hash_password},
    Result_type, Users_folder_path,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User_type {
    Identifier: User_identifier_inner_type,
    Name: String,
    Primary_group: Group_identifier_inner_type,
    Hash: String,
    Salt: String,
}

impl User_type {
    pub fn New(
        Identifier: User_identifier_inner_type,
        Name: String,
        Primary_group: Group_identifier_inner_type,
        Hash: String,
        Salt: String,
    ) -> Self {
        Self {
            Identifier,
            Name,
            Primary_group,
            Hash,
            Salt,
        }
    }

    pub fn Get_identifier(&self) -> User_identifier_type {
        User_identifier_type::New(self.Identifier)
    }

    pub fn Get_primary_group(&self) -> Group_identifier_type {
        Group_identifier_type::New(self.Primary_group)
    }

    pub fn Get_name(&self) -> &str {
        &self.Name
    }

    pub fn Get_hash(&self) -> &str {
        &self.Hash
    }

    pub fn Get_salt(&self) -> &str {
        &self.Salt
    }

    pub fn Set_hash(&mut self, Hash: String) {
        self.Hash = Hash;
    }

    pub fn Set_salt(&mut self, Salt: String) {
        self.Salt = Salt;
    }

    pub fn Set_primary_group(&mut self, Primary_group: Group_identifier_inner_type) {
        self.Primary_group = Primary_group;
    }

    pub fn Set_name(&mut self, Name: String) {
        self.Name = Name;
    }
}

pub fn Get_user_file_path(User_name: &str) -> Result_type<Path_owned_type> {
    Path_type::New(Users_folder_path)
        .to_owned()
        .Append(User_name)
        .ok_or(Error_type::Failed_to_get_user_file_path)
}

pub async fn Authenticate_user<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    User_name: &str,
    Password: &str,
) -> Result_type<User_identifier_type> {
    let Path = Get_user_file_path(User_name)?;

    let User_file = File_type::Open(Virtual_file_system, Path, Mode_type::Read_only.into())
        .await
        .map_err(Error_type::Failed_to_open_user_file)?;

    let mut Buffer = Vec::new();

    User_file
        .Read_to_end(&mut Buffer)
        .await
        .map_err(Error_type::Failed_to_read_user_file)?;

    let User: User_type = miniserde::json::from_str(core::str::from_utf8(&Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)?;

    if Hash_password(Password, User.Get_salt()) == User.Get_hash() {
        Ok(User.Get_identifier())
    } else {
        Err(Error_type::Invalid_password)
    }
}

pub async fn Create_user<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    User_name: &str,
    Password: &str,
    Primary_group: Group_identifier_type,
    User_identifier: Option<User_identifier_type>,
) -> Result_type<User_identifier_type> {
    let Users_manager = Users::Get_instance().await;

    // - New user identifier if not provided.
    let User_identifier = if let Some(User_identifier) = User_identifier {
        User_identifier
    } else {
        Users_manager
            .Get_new_user_identifier()
            .await
            .map_err(Error_type::Failed_to_get_new_user_identifier)?
    };

    // - Add it to the users manager.
    Users_manager
        .Add_user(User_identifier, User_name, Primary_group)
        .await
        .map_err(Error_type::Failed_to_create_user)?;

    // - Hash password.
    let Salt = Generate_salt().await?;

    let Hash = Hash_password(Password, &Salt);

    // - Write user file.
    let User = User_type::New(
        User_identifier.As_u16(),
        User_name.to_string(),
        Primary_group.As_u16(),
        Hash,
        Salt,
    );

    match Directory_type::Create(Virtual_file_system, Users_folder_path).await {
        Ok(_) | Err(File_system::Error_type::Already_exists) => {}
        Err(Error) => Err(Error_type::Failed_to_create_users_directory(Error))?,
    }

    let User_file_path = Path_type::New(Users_folder_path)
        .to_owned()
        .Append(User_name)
        .ok_or(Error_type::Failed_to_get_user_file_path)?;

    let User_file = File_type::Open(
        Virtual_file_system,
        User_file_path,
        Flags_type::New(Mode_type::Write_only, Some(Open_type::Create_only), None),
    )
    .await
    .map_err(Error_type::Failed_to_open_user_file)?;

    let User_json = miniserde::json::to_string(&User);

    User_file
        .Write(User_json.as_bytes())
        .await
        .map_err(Error_type::Failed_to_write_user_file)?;

    Ok(User_identifier)
}

pub async fn Change_user_password<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    User_name: &str,
    New_password: &str,
) -> Result_type<()> {
    let Salt = Generate_salt().await?;

    let Hash = Hash_password(New_password, &Salt);

    let User_file_path = Path_type::New(Users_folder_path)
        .to_owned()
        .Append(User_name)
        .ok_or(Error_type::Failed_to_get_user_file_path)?;

    let User_file = File_type::Open(
        Virtual_file_system,
        User_file_path,
        Flags_type::New(Mode_type::Read_write, Some(Open_type::Truncate), None),
    )
    .await
    .map_err(Error_type::Failed_to_open_user_file)?;

    let mut Buffer = Vec::new();

    User_file
        .Read_to_end(&mut Buffer)
        .await
        .map_err(Error_type::Failed_to_read_user_file)?;

    let mut User: User_type = miniserde::json::from_str(core::str::from_utf8(&Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)?;

    User.Set_hash(Hash);
    User.Set_salt(Salt);

    let User_json = miniserde::json::to_string(&User);

    User_file
        .Write(User_json.as_bytes())
        .await
        .map_err(Error_type::Failed_to_write_user_file)?;

    Ok(())
}

pub async fn Change_user_name<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Current_name: &str,
    New_name: &str,
) -> Result_type<()> {
    let File_path = Get_user_file_path(Current_name)?;

    let User_file = File_type::Open(
        Virtual_file_system,
        File_path,
        Flags_type::New(Mode_type::Read_write, Some(Open_type::Truncate), None),
    )
    .await
    .map_err(Error_type::Failed_to_open_user_file)?;

    let mut Buffer = Vec::new();

    User_file
        .Read_to_end(&mut Buffer)
        .await
        .map_err(Error_type::Failed_to_read_user_file)?;

    let mut User: User_type = miniserde::json::from_str(core::str::from_utf8(&Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)?;

    User.Set_name(New_name.to_string());

    let User_json = miniserde::json::to_string(&User);

    User_file
        .Write(User_json.as_bytes())
        .await
        .map_err(Error_type::Failed_to_write_user_file)?;

    Ok(())
}

pub async fn Read_user_file<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Buffer: &mut Vec<u8>,
    File: &str,
) -> Result_type<User_type> {
    let User_file_path = Get_user_file_path(File)?;

    let User_file = File_type::Open(
        Virtual_file_system,
        User_file_path,
        Mode_type::Read_only.into(),
    )
    .await
    .map_err(Error_type::Failed_to_read_users_directory)?;

    Buffer.clear();

    User_file
        .Read_to_end(Buffer)
        .await
        .map_err(Error_type::Failed_to_read_user_file)?;

    miniserde::json::from_str(core::str::from_utf8(Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)
}
