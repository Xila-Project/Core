#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

mod Error;
mod Group;
mod User;

pub use Error::*;
use File_system::{Flags_type, Mode_type, Open_type, Path_owned_type, Path_type};
use Users::{Group_identifier_type, User_identifier_type};
use Virtual_file_system::{Directory_type, File_type, Virtual_file_system_type};

const Users_folder_path: &str = "/Xila/Users";
const Group_folder_path: &str = "/Xila/Groups";
const Random_device_path: &str = "/Devices/Random";

pub fn Get_user_file_path(User_name: &str) -> Result_type<Path_owned_type> {
    Path_type::New(Users_folder_path)
        .to_owned()
        .Append(User_name)
        .ok_or(Error_type::Failed_to_get_user_file_path)
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
) -> Result_type<Group::Group_type> {
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

pub fn Read_user_file<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Buffer: &mut Vec<u8>,
    File: &str,
) -> Result_type<User::User_type> {
    let User_file_path = Get_user_file_path(File)?;

    let User_file = File_type::Open(
        Virtual_file_system,
        User_file_path,
        Mode_type::Read_only.into(),
    )
    .map_err(Error_type::Failed_to_read_users_directory)?;

    Buffer.clear();

    User_file
        .Read_to_end(Buffer)
        .map_err(Error_type::Failed_to_read_user_file)?;

    miniserde::json::from_str(core::str::from_utf8(Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)
}

pub fn Load_all_users_and_groups() -> Result_type<()> {
    // Open Xila users folder.
    let Virtual_file_system = Virtual_file_system::Get_instance();

    let Users_manager = Users::Get_instance();

    let mut Buffer: Vec<u8> = vec![];

    {
        let Groups_directory = Directory_type::Open(Virtual_file_system, Group_folder_path)
            .map_err(Error_type::Failed_to_read_group_directory)?;

        // Read all groups.
        for Group_entry in Groups_directory {
            let Group = if let Ok(Group) =
                Read_group_file(Virtual_file_system, &mut Buffer, Group_entry.Get_name())
            {
                Group
            } else {
                // ? : Log error ?
                continue;
            };

            Users_manager
                .Add_group(Group.Get_identifier(), Group.Get_name(), Group.Get_users())
                .map_err(Error_type::Failed_to_add_group)?;
        }
    }

    {
        let Users_directory = Directory_type::Open(Virtual_file_system, Users_folder_path)
            .map_err(Error_type::Failed_to_read_users_directory)?;

        // Read all users.
        for User_entry in Users_directory {
            let User = if let Ok(User) =
                Read_user_file(Virtual_file_system, &mut Buffer, User_entry.Get_name())
            {
                User
            } else {
                // ? : Log error ?
                continue;
            };

            Users_manager
                .Add_user(
                    User.Get_identifier(),
                    User.Get_name(),
                    User.Get_primary_group(),
                )
                .map_err(Error_type::Failed_to_add_user)?;
        }
    }

    Ok(())
}

pub fn Generate_salt() -> Result_type<String> {
    let Random_file = File_type::Open(
        Virtual_file_system::Get_instance(),
        Random_device_path,
        Mode_type::Read_only.into(),
    )
    .map_err(Error_type::Failed_to_open_random_device)?;

    let mut Buffer = [0_u8; 16];

    Random_file
        .Read(&mut Buffer)
        .map_err(Error_type::Failed_to_read_random_device)?;

    Buffer.iter_mut().for_each(|Byte| {
        *Byte = *Byte % 26 + 97;
    });

    Ok(core::str::from_utf8(&Buffer).unwrap().to_string())
}

pub fn Hash_password(Password: &str, Salt: &str) -> String {
    use sha2::Digest;

    let mut Hasher = sha2::Sha512::new();

    Hasher.update(Password.as_bytes());
    Hasher.update(Salt.as_bytes());

    let Hash = Hasher.finalize();

    format!("{:x}", Hash)
}

pub fn Authenticate_user<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    User_name: &str,
    Password: &str,
) -> Result_type<User_identifier_type> {
    let Path = Get_user_file_path(User_name)?;

    let User_file = File_type::Open(Virtual_file_system, Path, Mode_type::Read_only.into())
        .map_err(Error_type::Failed_to_open_user_file)?;

    let mut Buffer = Vec::new();

    User_file
        .Read_to_end(&mut Buffer)
        .map_err(Error_type::Failed_to_read_user_file)?;

    let User: User::User_type = miniserde::json::from_str(core::str::from_utf8(&Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)?;

    if Hash_password(Password, User.Get_salt()) == User.Get_hash() {
        Ok(User.Get_identifier())
    } else {
        Err(Error_type::Invalid_password)
    }
}

pub fn Create_user<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    User_name: &str,
    Password: &str,
    Primary_group: Group_identifier_type,
    User_identifier: Option<User_identifier_type>,
) -> Result_type<User_identifier_type> {
    let Users_manager = Users::Get_instance();

    // - New user identifier if not provided.
    let User_identifier = if let Some(User_identifier) = User_identifier {
        User_identifier
    } else {
        Users_manager
            .Get_new_user_identifier()
            .map_err(Error_type::Failed_to_get_new_user_identifier)?
    };

    // - Add it to the users manager.
    Users_manager
        .Add_user(User_identifier, User_name, Primary_group)
        .map_err(Error_type::Failed_to_create_user)?;

    // - Hash password.
    let Salt = Generate_salt()?;

    let Hash = Hash_password(Password, &Salt);

    // - Write user file.
    let User = User::User_type::New(
        User_identifier.Into_inner(),
        User_name.to_string(),
        Primary_group.Into_inner(),
        Hash,
        Salt,
    );

    let User_file_path = Path_type::New(Users_folder_path)
        .to_owned()
        .Append(User_name)
        .ok_or(Error_type::Failed_to_get_user_file_path)?;

    let User_file = File_type::Open(
        Virtual_file_system,
        User_file_path,
        Flags_type::New(Mode_type::Write_only, Some(Open_type::Create_only), None),
    )
    .map_err(Error_type::Failed_to_open_user_file)?;

    let User_json = miniserde::json::to_string(&User);

    User_file
        .Write(User_json.as_bytes())
        .map_err(Error_type::Failed_to_write_user_file)?;

    Ok(User_identifier)
}

pub fn Change_user_password<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    User_name: &str,
    New_password: &str,
) -> Result_type<()> {
    let Salt = Generate_salt()?;

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
    .map_err(Error_type::Failed_to_open_user_file)?;

    let mut Buffer = Vec::new();

    User_file
        .Read_to_end(&mut Buffer)
        .map_err(Error_type::Failed_to_read_user_file)?;

    let mut User: User::User_type =
        miniserde::json::from_str(core::str::from_utf8(&Buffer).unwrap())
            .map_err(Error_type::Failed_to_parse_user_file)?;

    User.Set_hash(Hash);
    User.Set_salt(Salt);

    let User_json = miniserde::json::to_string(&User);

    User_file
        .Write(User_json.as_bytes())
        .map_err(Error_type::Failed_to_write_user_file)?;

    Ok(())
}

pub fn Change_user_name<'a>(
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
    .map_err(Error_type::Failed_to_open_user_file)?;

    let mut Buffer = Vec::new();

    User_file
        .Read_to_end(&mut Buffer)
        .map_err(Error_type::Failed_to_read_user_file)?;

    let mut User: User::User_type =
        miniserde::json::from_str(core::str::from_utf8(&Buffer).unwrap())
            .map_err(Error_type::Failed_to_parse_user_file)?;

    User.Set_name(New_name.to_string());

    let User_json = miniserde::json::to_string(&User);

    User_file
        .Write(User_json.as_bytes())
        .map_err(Error_type::Failed_to_write_user_file)?;

    Ok(())
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
    let Group = Group::Group_type::New(
        Group_identifier.Into_inner(),
        Group_name.to_string(),
        vec![],
    );

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
