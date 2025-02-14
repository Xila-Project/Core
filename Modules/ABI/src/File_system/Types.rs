use File_system::{Position_type, Type_type};

use crate::{Xila_group_identifier_type, Xila_time_type, Xila_user_identifier_type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Whence_type {
    Start,
    Current,
    End,
}

pub const fn Into_position(Whence: Whence_type, Offset: i64) -> Position_type {
    match Whence {
        Whence_type::Start => Position_type::Start(Offset as u64),
        Whence_type::Current => Position_type::Current(Offset),
        Whence_type::End => Position_type::End(Offset),
    }
}

#[repr(u8)]
pub enum Xila_type_type {
    File,
    Directory,
    Block_device,
    Character_device,
    Pipe,
    Socket,
    Symbolic_link,
}

impl From<Xila_type_type> for File_system::Type_type {
    fn from(Type: Xila_type_type) -> Self {
        match Type {
            Xila_type_type::File => Type_type::File,
            Xila_type_type::Directory => File_system::Type_type::Directory,
            Xila_type_type::Block_device => File_system::Type_type::Block_device,
            Xila_type_type::Character_device => File_system::Type_type::Character_device,
            Xila_type_type::Pipe => File_system::Type_type::Pipe,
            Xila_type_type::Socket => File_system::Type_type::Socket,
            Xila_type_type::Symbolic_link => File_system::Type_type::Symbolic_link,
        }
    }
}

impl From<File_system::Type_type> for Xila_type_type {
    fn from(Type: File_system::Type_type) -> Self {
        match Type {
            File_system::Type_type::File => Xila_type_type::File,
            File_system::Type_type::Directory => Xila_type_type::Directory,
            File_system::Type_type::Block_device => Xila_type_type::Block_device,
            File_system::Type_type::Character_device => Xila_type_type::Character_device,
            File_system::Type_type::Pipe => Xila_type_type::Pipe,
            File_system::Type_type::Socket => Xila_type_type::Socket,
            File_system::Type_type::Symbolic_link => Xila_type_type::Symbolic_link,
        }
    }
}

pub type Xila_mode_type = u8;

#[no_mangle]
pub static Xila_file_system_mode_read_mask: u8 = File_system::Mode_type::Read_bit;
#[no_mangle]
pub static Xila_file_system_mode_write_mask: u8 = File_system::Mode_type::Write_bit;

pub type Xila_open_type = u8;

#[no_mangle]
pub static Xila_file_system_open_create_mask: u8 = File_system::Open_type::Create_mask;
#[no_mangle]
pub static Xila_file_system_open_create_only_mask: u8 = File_system::Open_type::Exclusive_mask;
#[no_mangle]
pub static Xila_file_system_open_truncate_mask: u8 = File_system::Open_type::Truncate_mask;

pub type Xila_status_type = u8;

#[no_mangle]
pub static Xila_file_system_status_append_mask: u8 = File_system::Status_type::Append_bit;
#[no_mangle]
pub static Xila_file_system_status_non_blocking_mask: u8 =
    File_system::Status_type::Non_blocking_bit;
#[no_mangle]
pub static Xila_file_system_status_synchronous_mask: u8 = File_system::Status_type::Synchronous_bit;
#[no_mangle]
pub static Xila_file_system_status_synchronous_data_only_mask: u8 =
    File_system::Status_type::Synchronous_data_only_bit;

pub type Xila_inode_type = u64;

#[cfg(target_pointer_width = "64")]
type File_system_identifier_type = u32;
#[cfg(target_pointer_width = "32")]
type File_system_identifier_type = u16;

type Permissions_type = u16;

#[repr(C)]
pub struct Xila_statistics_type {
    File_system: File_system_identifier_type,
    Inode: Xila_inode_type,
    Links: u64,
    Size: Xila_size_type,
    Last_access: Xila_time_type,
    Last_modification: Xila_time_type,
    Last_status_change: Xila_time_type,
    Type: Xila_type_type,
    Permissions: Permissions_type,
    User: Xila_user_identifier_type,
    Group: Xila_group_identifier_type,
}

impl Xila_statistics_type {
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
        Pointer: *mut Xila_statistics_type,
    ) -> Option<*mut Xila_statistics_type> {
        if Pointer.is_null() {
            return None;
        }

        if Pointer as usize % std::mem::align_of::<Xila_statistics_type>() != 0 {
            return None;
        }

        Some(Pointer)
    }
}

pub type Xila_unique_file_identifier_type = usize;
pub type Xila_size_type = u64;
