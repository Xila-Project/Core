use Task::Task_identifier_type;
use Users::{Group_identifier_type, User_identifier_type};

use crate::{Flags_type, Permissions_type, Time_type, Type_type};

use super::{Error_type, Result_type};

/// File attributes.
///
/// The attributes are metadata associated with the file that stores:
/// - The file type.
/// - The file creation time.
/// - The file modification time.
/// - The file access time.
/// - The file permissions.
/// - The file owner.
/// - The file group.
#[derive(Debug, Clone)]
pub struct Metadata_type {
    /// The file type.
    pub Type: Type_type,
    /// The file creation time.
    pub Creation_time: Time_type,
    /// The file modification time.
    pub Modification_time: Time_type,
    /// The file access time.
    pub Access_time: Time_type,
    /// The file permissions.
    pub Permissions: Permissions_type,
    /// The file owner.
    pub Owner: User_identifier_type,
    /// The file group.
    pub Group: Group_identifier_type,
}

impl Metadata_type {
    pub const Identifer: u8 = 0x01;

    pub const Default: Self = Self {
        Type: Type_type::File,
        Creation_time: Time_type::New(0),
        Modification_time: Time_type::New(0),
        Access_time: Time_type::New(0),
        Permissions: Permissions_type::None,
        Owner: Users::Root_user_identifier,
        Group: Users::Root_user_identifier,
    };

    pub fn Get(Task: Task_identifier_type, Flags: Flags_type) -> Result_type<Self> {
        let Users_instance = Users::Get_instance();
        let Task_instance = Task::Get_instance();

        let Owner = Task_instance
            .Get_owner(Task)
            .map_err(|_| Error_type::Invalid_parameter)?;
        let Group = Users_instance
            .Get_user_primary_group(Owner)
            .map_err(|_| Error_type::Invalid_parameter)?;

        let Type = if Flags.Get_open().Get_directory() {
            Type_type::Directory
        } else {
            Type_type::File
        };

        let Permissions = Permissions_type::New_default(Type);

        let Current_time = Time_type::Get_now();

        Ok(Metadata_type {
            Type,
            Creation_time: Current_time,
            Modification_time: Current_time,
            Access_time: Current_time,
            Permissions,
            Owner,
            Group,
        })
    }
}

impl AsRef<[u8]> for Metadata_type {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const _ as *const u8,
                core::mem::size_of::<Metadata_type>(),
            )
        }
    }
}

impl AsMut<[u8]> for Metadata_type {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self as *mut _ as *mut u8,
                core::mem::size_of::<Metadata_type>(),
            )
        }
    }
}
