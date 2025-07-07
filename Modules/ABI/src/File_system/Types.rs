use File_system::{Position_type, Type_type};

use crate::{Xila_group_identifier_type, Xila_time_type, Xila_user_identifier_type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Xila_file_system_whence_type {
    Start,
    Current,
    End,
}

pub const fn Into_position(Whence: Xila_file_system_whence_type, Offset: i64) -> Position_type {
    match Whence {
        Xila_file_system_whence_type::Start => Position_type::Start(Offset as u64),
        Xila_file_system_whence_type::Current => Position_type::Current(Offset),
        Xila_file_system_whence_type::End => Position_type::End(Offset),
    }
}

#[repr(u8)]
pub enum Xila_file_type_type {
    File,
    Directory,
    Block_device,
    Character_device,
    Pipe,
    Socket,
    Symbolic_link,
}

impl From<Xila_file_type_type> for File_system::Type_type {
    fn from(Type: Xila_file_type_type) -> Self {
        match Type {
            Xila_file_type_type::File => Type_type::File,
            Xila_file_type_type::Directory => File_system::Type_type::Directory,
            Xila_file_type_type::Block_device => File_system::Type_type::Block_device,
            Xila_file_type_type::Character_device => File_system::Type_type::Character_device,
            Xila_file_type_type::Pipe => File_system::Type_type::Pipe,
            Xila_file_type_type::Socket => File_system::Type_type::Socket,
            Xila_file_type_type::Symbolic_link => File_system::Type_type::Symbolic_link,
        }
    }
}

impl From<File_system::Type_type> for Xila_file_type_type {
    fn from(Type: File_system::Type_type) -> Self {
        match Type {
            File_system::Type_type::File => Xila_file_type_type::File,
            File_system::Type_type::Directory => Xila_file_type_type::Directory,
            File_system::Type_type::Block_device => Xila_file_type_type::Block_device,
            File_system::Type_type::Character_device => Xila_file_type_type::Character_device,
            File_system::Type_type::Pipe => Xila_file_type_type::Pipe,
            File_system::Type_type::Socket => Xila_file_type_type::Socket,
            File_system::Type_type::Symbolic_link => Xila_file_type_type::Symbolic_link,
        }
    }
}

pub type Xila_file_system_mode_type = u8;

#[no_mangle]
pub static XILA_FILE_SYSTEM_MODE_READ_MASK: u8 = File_system::Mode_type::READ_BIT;
#[no_mangle]
pub static XILA_FILE_SYSTEM_MODE_WRITE_MASK: u8 = File_system::Mode_type::WRITE_BIT;

pub type Xila_file_system_open_type = u8;

#[no_mangle]
pub static XILA_FILE_SYSTEM_OPEN_CREATE_MASK: u8 = File_system::Open_type::CREATE_MASK;
#[no_mangle]
pub static XILA_FILE_SYSTEM_OPEN_CREATE_ONLY_MASK: u8 = File_system::Open_type::EXCLUSIVE_MASK;
#[no_mangle]
pub static XILA_FILE_SYSTEM_OPEN_TRUNCATE_MASK: u8 = File_system::Open_type::TRUNCATE_MASK;

pub type Xila_file_system_status_type = u8;

#[no_mangle]
pub static XILA_FILE_SYSTEM_STATUS_APPEND_MASK: u8 = File_system::Status_type::APPEND_BIT;
#[no_mangle]
pub static XILA_FILE_SYSTEM_STATUS_NON_BLOCKING_MASK: u8 =
    File_system::Status_type::NON_BLOCKING_BIT;
#[no_mangle]
pub static XILA_FILE_SYSTEM_STATUS_SYNCHRONOUS_MASK: u8 = File_system::Status_type::SYNCHRONOUS_BIT;
#[no_mangle]
pub static XILA_FILE_SYSTEM_STATUS_SYNCHRONOUS_DATA_ONLY_MASK: u8 =
    File_system::Status_type::SYNCHRONOUS_DATA_ONLY_BIT;

pub type Xila_file_system_inode_type = u64;

type Xila_file_system_identifier_type = u32;

type Permissions_type = u16;

#[repr(C)]
pub struct Xila_file_system_statistics_type {
    File_system: Xila_file_system_identifier_type,
    Inode: Xila_file_system_inode_type,
    Links: u64,
    Size: Xila_file_system_size_type,
    Last_access: Xila_time_type,
    Last_modification: Xila_time_type,
    Last_status_change: Xila_time_type,
    Type: Xila_file_type_type,
    Permissions: Permissions_type,
    User: Xila_user_identifier_type,
    Group: Xila_group_identifier_type,
}

impl Xila_file_system_statistics_type {
    pub fn From_statistics(Statistics: File_system::Statistics_type) -> Self {
        Self {
            File_system: Statistics.Get_file_system().As_inner(),
            Inode: Statistics.Get_inode().As_u64(),
            Links: Statistics.Get_links(),
            Size: Statistics.Get_size().As_u64(),
            Last_access: Statistics.Get_last_access().As_u64(),
            Last_modification: Statistics.Get_last_modification().As_u64(),
            Last_status_change: Statistics.Get_last_status_change().As_u64(),
            Type: Statistics.Get_type().into(),
            Permissions: Statistics.Get_permissions().As_u16(),
            User: Statistics.Get_user().As_u16(),
            Group: Statistics.Get_group().As_u16(),
        }
    }

    pub fn From_mutable_pointer(
        Pointer: *mut Xila_file_system_statistics_type,
    ) -> Option<*mut Xila_file_system_statistics_type> {
        if Pointer.is_null() {
            return None;
        }

        if Pointer as usize % align_of::<Xila_file_system_statistics_type>() != 0 {
            return None;
        }

        Some(Pointer)
    }
}

pub type Xila_unique_file_identifier_type = usize;
pub type Xila_file_system_size_type = u64;

pub type Xila_file_system_result_type = u32;
