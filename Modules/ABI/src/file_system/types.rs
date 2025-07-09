use file_system::{Position_type, Type_type};

use crate::{Xila_group_identifier_type, Xila_time_type, Xila_user_identifier_type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Xila_file_system_whence_type {
    Start,
    Current,
    End,
}

pub const fn into_position(whence: Xila_file_system_whence_type, offset: i64) -> Position_type {
    match whence {
        Xila_file_system_whence_type::Start => Position_type::Start(offset as u64),
        Xila_file_system_whence_type::Current => Position_type::Current(offset),
        Xila_file_system_whence_type::End => Position_type::End(offset),
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

impl From<Xila_file_type_type> for file_system::Type_type {
    fn from(type_value: Xila_file_type_type) -> Self {
        match type_value {
            Xila_file_type_type::File => Type_type::File,
            Xila_file_type_type::Directory => file_system::Type_type::Directory,
            Xila_file_type_type::Block_device => file_system::Type_type::Block_device,
            Xila_file_type_type::Character_device => file_system::Type_type::Character_device,
            Xila_file_type_type::Pipe => file_system::Type_type::Pipe,
            Xila_file_type_type::Socket => file_system::Type_type::Socket,
            Xila_file_type_type::Symbolic_link => file_system::Type_type::Symbolic_link,
        }
    }
}

impl From<file_system::Type_type> for Xila_file_type_type {
    fn from(type_value: file_system::Type_type) -> Self {
        match type_value {
            file_system::Type_type::File => Xila_file_type_type::File,
            file_system::Type_type::Directory => Xila_file_type_type::Directory,
            file_system::Type_type::Block_device => Xila_file_type_type::Block_device,
            file_system::Type_type::Character_device => Xila_file_type_type::Character_device,
            file_system::Type_type::Pipe => Xila_file_type_type::Pipe,
            file_system::Type_type::Socket => Xila_file_type_type::Socket,
            file_system::Type_type::Symbolic_link => Xila_file_type_type::Symbolic_link,
        }
    }
}

pub type Xila_file_system_mode_type = u8;

#[no_mangle]
pub static XILA_FILE_SYSTEM_MODE_READ_MASK: u8 = file_system::Mode_type::READ_BIT;
#[no_mangle]
pub static XILA_FILE_SYSTEM_MODE_WRITE_MASK: u8 = file_system::Mode_type::WRITE_BIT;

pub type Xila_file_system_open_type = u8;

#[no_mangle]
pub static XILA_FILE_SYSTEM_OPEN_CREATE_MASK: u8 = file_system::Open_type::CREATE_MASK;
#[no_mangle]
pub static XILA_FILE_SYSTEM_OPEN_CREATE_ONLY_MASK: u8 = file_system::Open_type::EXCLUSIVE_MASK;
#[no_mangle]
pub static XILA_FILE_SYSTEM_OPEN_TRUNCATE_MASK: u8 = file_system::Open_type::TRUNCATE_MASK;

pub type Xila_file_system_status_type = u8;

#[no_mangle]
pub static XILA_FILE_SYSTEM_STATUS_APPEND_MASK: u8 = file_system::Status_type::APPEND_BIT;
#[no_mangle]
pub static XILA_FILE_SYSTEM_STATUS_NON_BLOCKING_MASK: u8 =
    file_system::Status_type::NON_BLOCKING_BIT;
#[no_mangle]
pub static XILA_FILE_SYSTEM_STATUS_SYNCHRONOUS_MASK: u8 = file_system::Status_type::SYNCHRONOUS_BIT;
#[no_mangle]
pub static XILA_FILE_SYSTEM_STATUS_SYNCHRONOUS_DATA_ONLY_MASK: u8 =
    file_system::Status_type::SYNCHRONOUS_DATA_ONLY_BIT;

pub type Xila_file_system_inode_type = u64;

type Xila_file_system_identifier_type = u32;

type Permissions_type = u16;

#[repr(C)]
pub struct Xila_file_system_statistics_type {
    file_system: Xila_file_system_identifier_type,
    inode: Xila_file_system_inode_type,
    links: u64,
    size: Xila_file_system_size_type,
    last_access: Xila_time_type,
    last_modification: Xila_time_type,
    last_status_change: Xila_time_type,
    r#type: Xila_file_type_type,
    permissions: Permissions_type,
    user: Xila_user_identifier_type,
    group: Xila_group_identifier_type,
}

impl Xila_file_system_statistics_type {
    pub fn from_statistics(statistics: file_system::Statistics_type) -> Self {
        Self {
            file_system: statistics.get_file_system().As_inner(),
            inode: statistics.get_inode().As_u64(),
            links: statistics.get_links(),
            size: statistics.get_size().As_u64(),
            last_access: statistics.get_last_access().as_u64(),
            last_modification: statistics.get_last_modification().as_u64(),
            last_status_change: statistics.get_last_status_change().as_u64(),
            r#type: statistics.get_type().into(),
            permissions: statistics.get_permissions().As_u16(),
            user: statistics.get_user().As_u16(),
            group: statistics.get_group().As_u16(),
        }
    }

    pub fn from_mutable_pointer(
        pointer: *mut Xila_file_system_statistics_type,
    ) -> Option<*mut Xila_file_system_statistics_type> {
        if pointer.is_null() {
            return None;
        }

        if pointer as usize % align_of::<Xila_file_system_statistics_type>() != 0 {
            return None;
        }

        Some(pointer)
    }
}

pub type Xila_unique_file_identifier_type = usize;
pub type Xila_file_system_size_type = u64;

pub type Xila_file_system_result_type = u32;
