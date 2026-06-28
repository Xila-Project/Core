use alloc::vec::Vec;
use xila::virtual_file_system::{SynchronousDirectory, SynchronousFile};

pub struct WasiContext {
    pub fds: Vec<FileDescriptor>,
    pub next_fd: u32,
    pub args: Vec<Vec<u8>>,
    pub env: Vec<(Vec<u8>, Vec<u8>)>,
    pub random_state: u64,
}

pub struct FileDescriptor {
    pub fd: i32,
    pub ty: FdType,
    pub offset: u64,
    pub rights: u64,
}

pub enum FdType {
    File(SynchronousFile),
    Directory(SynchronousDirectory),
}
