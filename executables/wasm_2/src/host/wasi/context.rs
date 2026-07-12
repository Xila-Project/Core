use alloc::vec::Vec;
use xila::{
    task::TaskIdentifier,
    virtual_file_system::{SynchronousDirectory, SynchronousFile},
};

pub struct WasiContext {
    pub fds: Vec<FileDescriptor>,
    pub next_fd: u32,
    pub args: Vec<Vec<u8>>,
    pub task: TaskIdentifier,
    pub random_state: u64,
    pub prestats: Vec<Prestat>,
    pub exit_code: Option<i32>,
}

pub struct Prestat {
    pub name: Vec<u8>,
}

pub struct FileDescriptor {
    pub fd: i32,
    pub ty: FdType,
    pub offset: u64,
    pub rights: u64,
    pub rights_inheriting: u64,
    pub flags: u16,
}

pub enum FdType {
    File(SynchronousFile),
    Directory(SynchronousDirectory, Vec<u8>),
    CharacterDevice,
    Stdout(SynchronousFile),
    Stderr(SynchronousFile),
}
