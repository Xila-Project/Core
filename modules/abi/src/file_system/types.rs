use file_system::{Kind, Position};

use crate::{XilaGroupIdentifier, XilaTime, XilaUserIdentifier};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum XilaFileSystemWhence {
    Start,
    Current,
    End,
}

pub const fn into_position(whence: XilaFileSystemWhence, offset: i64) -> Position {
    match whence {
        XilaFileSystemWhence::Start => Position::Start(offset as u64),
        XilaFileSystemWhence::Current => Position::Current(offset),
        XilaFileSystemWhence::End => Position::End(offset),
    }
}

#[repr(u8)]
pub enum XilaFileKind {
    File,
    Directory,
    BlockDevice,
    CharacterDevice,
    Pipe,
    Socket,
    SymbolicLink,
}

impl From<XilaFileKind> for file_system::Kind {
    fn from(type_value: XilaFileKind) -> Self {
        match type_value {
            XilaFileKind::File => Kind::File,
            XilaFileKind::Directory => file_system::Kind::Directory,
            XilaFileKind::BlockDevice => file_system::Kind::BlockDevice,
            XilaFileKind::CharacterDevice => file_system::Kind::CharacterDevice,
            XilaFileKind::Pipe => file_system::Kind::Pipe,
            XilaFileKind::Socket => file_system::Kind::Socket,
            XilaFileKind::SymbolicLink => file_system::Kind::SymbolicLink,
        }
    }
}

impl From<file_system::Kind> for XilaFileKind {
    fn from(type_value: file_system::Kind) -> Self {
        match type_value {
            file_system::Kind::File => XilaFileKind::File,
            file_system::Kind::Directory => XilaFileKind::Directory,
            file_system::Kind::BlockDevice => XilaFileKind::BlockDevice,
            file_system::Kind::CharacterDevice => XilaFileKind::CharacterDevice,
            file_system::Kind::Pipe => XilaFileKind::Pipe,
            file_system::Kind::Socket => XilaFileKind::Socket,
            file_system::Kind::SymbolicLink => XilaFileKind::SymbolicLink,
        }
    }
}

pub type XilaFileSystemMode = u8;

#[unsafe(no_mangle)]
pub static XILA_FILE_SYSTEM_MODE_READ_MASK: u8 = file_system::Mode::READ_BIT;
#[unsafe(no_mangle)]
pub static XILA_FILE_SYSTEM_MODE_WRITE_MASK: u8 = file_system::Mode::WRITE_BIT;

pub type XilaFileSystemOpen = u8;

#[unsafe(no_mangle)]
pub static XILA_FILE_SYSTEM_OPEN_CREATE_MASK: u8 = file_system::Open::CREATE_MASK;
#[unsafe(no_mangle)]
pub static XILA_FILE_SYSTEM_OPEN_CREATE_ONLY_MASK: u8 = file_system::Open::EXCLUSIVE_MASK;
#[unsafe(no_mangle)]
pub static XILA_FILE_SYSTEM_OPEN_TRUNCATE_MASK: u8 = file_system::Open::TRUNCATE_MASK;

pub type XilaFileSystemStatus = u8;

#[unsafe(no_mangle)]
pub static XILA_FILE_SYSTEM_STATUS_APPEND_MASK: u8 = file_system::Status::APPEND_BIT;
#[unsafe(no_mangle)]
pub static XILA_FILE_SYSTEM_STATUS_NON_BLOCKING_MASK: u8 = file_system::Status::NON_BLOCKING_BIT;
#[unsafe(no_mangle)]
pub static XILA_FILE_SYSTEM_STATUS_SYNCHRONOUS_MASK: u8 = file_system::Status::SYNCHRONOUS_BIT;
#[unsafe(no_mangle)]
pub static XILA_FILE_SYSTEM_STATUS_SYNCHRONOUS_DATA_ONLY_MASK: u8 =
    file_system::Status::SYNCHRONOUS_DATA_ONLY_BIT;

pub type XilaFileSystemInode = u64;

type XilaFileSystemIdentifier = u32;

type Permissions = u16;

#[repr(C)]
pub struct XilaFileSystemStatistics {
    file_system: XilaFileSystemIdentifier,
    inode: XilaFileSystemInode,
    links: u64,
    size: XilaFileSystemSize,
    last_access: XilaTime,
    last_modification: XilaTime,
    last_status_change: XilaTime,
    r#type: XilaFileKind,
    permissions: Permissions,
    user: XilaUserIdentifier,
    group: XilaGroupIdentifier,
}

impl XilaFileSystemStatistics {
    pub fn from_statistics(statistics: file_system::Statistics_type) -> Self {
        Self {
            file_system: statistics.get_file_system().as_inner(),
            inode: statistics.get_inode().as_u64(),
            links: statistics.get_links(),
            size: statistics.get_size().as_u64(),
            last_access: statistics.get_last_access().as_u64(),
            last_modification: statistics.get_last_modification().as_u64(),
            last_status_change: statistics.get_last_status_change().as_u64(),
            r#type: statistics.get_type().into(),
            permissions: statistics.get_permissions().as_u16(),
            user: statistics.get_user().as_u16(),
            group: statistics.get_group().as_u16(),
        }
    }

    pub fn from_mutable_pointer(
        pointer: *mut XilaFileSystemStatistics,
    ) -> Option<*mut XilaFileSystemStatistics> {
        if pointer.is_null() {
            return None;
        }

        if !(pointer as usize).is_multiple_of(align_of::<XilaFileSystemStatistics>()) {
            return None;
        }

        Some(pointer)
    }
}

pub type XilaUniqueFileIdentifier = usize;
pub type XilaFileSystemSize = u64;

pub type XilaFileSystemResult = u32;
