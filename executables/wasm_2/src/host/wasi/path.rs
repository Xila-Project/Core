use wasmi::Caller;
use xila::file_system::{AccessFlags, CreateFlags, Flags, StateFlags};
use xila::task;

use crate::{
    define_wasi_module,
    host::{
        store::GlobalStore,
        wasi::{
            context::{FileDescriptor, FileSystemItem},
            error::vfs_error,
            memory::get_memory,
            types::Filestat,
        },
    },
};

fn kind_to_filetype(kind: &xila::file_system::Kind) -> u8 {
    use xila::file_system::Kind;
    match kind {
        Kind::Directory => 3,
        Kind::CharacterDevice => 2,
        Kind::BlockDevice => 1,
        Kind::Pipe => 10,
        Kind::Socket => 11,
        Kind::SymbolicLink => 7,
        _ => 4,
    }
}

define_wasi_module! {
    module: "wasi_snapshot_preview1";

    fn path_open(
        caller: Caller<GlobalStore>,
        fd: i32,
        _dirflags: i32,
        path_ptr: i32,
        path_len: i32,
        oflags: i32,
        fs_rights_base: i64,
        fs_rights_inheriting: i64,
        fdflags: i32,
        opened_fd_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        let mut caller = caller;
        let (rel_path_bytes, task, dir_path_bytes) = {
            let caller = &caller;
            let memory = get_memory(caller)?;
            let data = memory.data(caller);
            let rel = data[path_ptr as usize..path_ptr as usize + path_len as usize].to_vec();
            let store = caller.data();
            let entry = match store.wasi.files.iter().find(|e| e.fd == fd) {
                Some(e) => e,
                None => return Ok(8),
            };
            let dir = match &entry.ty {
                FileSystemItem::Directory(_, p) => p.clone(),
                _ => return Ok(8),
            };
            (rel, store.wasi.task, dir)
        };
        let rel_path = match core::str::from_utf8(&rel_path_bytes) {
            Ok(s) => s,
            Err(_) => return Ok(28),
        };
        let dir_path = match core::str::from_utf8(&dir_path_bytes) {
            Ok(s) => s,
            Err(_) => return Ok(28),
        };
        let abs_path = match xila::file_system::Path::from_str(dir_path).join(xila::file_system::Path::from_str(rel_path)) {
            Some(p) => p,
            None => return Ok(28),
        };

        let read = (fs_rights_base & 2) != 0;
        let write = (fs_rights_base & 32) != 0;
        let access = if read && write {
            AccessFlags::READ_WRITE
        } else if write {
            AccessFlags::Write
        } else {
            AccessFlags::Read
        };

        let create = if oflags & 1 != 0 {
            if oflags & 4 != 0 {
                Some(CreateFlags::CREATE_EXCLUSIVE)
            } else if oflags & 8 != 0 {
                Some(CreateFlags::CREATE_TRUNCATE)
            } else {
                Some(CreateFlags::Create)
            }
        } else {
            None
        };

        let state = if fdflags & 1 != 0 {
            Some(StateFlags::Append)
        } else {
            None
        };

        let vfs = xila::virtual_file_system::get_instance();

        if oflags & 2 != 0 {
            match xila::virtual_file_system::SynchronousDirectory::open(vfs, task, &abs_path) {
                Ok(dir) => {
                    let path_string = alloc::format!("{}", &abs_path).into_bytes();
                    let new_fd;
                    {
                        let store = caller.data_mut();
                        new_fd = store.wasi.next_fd as i32;
                        store.wasi.next_fd += 1;
                        store.wasi.files.push(FileDescriptor {
                            fd: new_fd,
                            ty: FileSystemItem::Directory(dir, path_string),
                            offset: 0,
                            rights: fs_rights_base as u64,
                            rights_inheriting: fs_rights_inheriting as u64,
                            flags: fdflags as u16,
                        });
                    }
                    let memory = get_memory(&caller)?;
                    let data = memory.data_mut(&mut caller);
                    data[opened_fd_ptr as usize..opened_fd_ptr as usize + 4].copy_from_slice(&new_fd.to_le_bytes());
                    Ok(0)
                }
                Err(e) => Ok(vfs_error(e) as i32),
            }
        } else {
            let xila_flags = Flags::new(access, create, state);
            match xila::virtual_file_system::SynchronousFile::open(vfs, task, &abs_path, xila_flags) {
                Ok(file) => {
                    let new_fd;
                    {
                        let store = caller.data_mut();
                        new_fd = store.wasi.next_fd as i32;
                        store.wasi.next_fd += 1;
                        store.wasi.files.push(FileDescriptor {
                            fd: new_fd,
                            ty: FileSystemItem::File(file),
                            offset: 0,
                            rights: fs_rights_base as u64,
                            rights_inheriting: fs_rights_inheriting as u64,
                            flags: fdflags as u16,
                        });
                    }
                    let memory = get_memory(&caller)?;
                    let data = memory.data_mut(&mut caller);
                    data[opened_fd_ptr as usize..opened_fd_ptr as usize + 4].copy_from_slice(&new_fd.to_le_bytes());
                    Ok(0)
                }
                Err(e) => Ok(vfs_error(e) as i32),
            }
        }
    }

    fn path_filestat_get(
        caller: Caller<GlobalStore>,
        fd: i32,
        _flags: i32,
        path_ptr: i32,
        path_len: i32,
        buf_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        xila::log::information!(
            "path_filestat_get: fd={}, path={:#x}, path_len={}, buffer={:#x}",
            fd, path_ptr, path_len, buf_ptr
        );
        let mut caller = caller;
        let (rel_path_bytes, _task_id, dir_path_bytes) = {
            let caller = &caller;
            let memory = get_memory(caller)?;
            let data = memory.data(caller);
            let rel = data[path_ptr as usize..path_ptr as usize + path_len as usize].to_vec();
            let store = caller.data();
            let entry = match store.wasi.files.iter().find(|e| e.fd == fd) {
                Some(e) => e,
                None => return Ok(8),
            };
            let dir = match &entry.ty {
                FileSystemItem::Directory(_, p) => p.clone(),
                _ => return Ok(8),
            };
            (rel, store.wasi.task, dir)
        };
        let rel_path = match core::str::from_utf8(&rel_path_bytes) {
            Ok(s) => s,
            Err(_) => return Ok(28),
        };
        let dir_path = match core::str::from_utf8(&dir_path_bytes) {
            Ok(s) => s,
            Err(_) => return Ok(28),
        };
        let abs_path = match xila::file_system::Path::from_str(dir_path).join(xila::file_system::Path::from_str(rel_path)) {
            Some(p) => p,
            None => return Ok(28),
        };

        let stats = match task::block_on(
            xila::virtual_file_system::get_instance().get_statistics(&abs_path)
        ) {
            Ok(s) => s,
            Err(e) => return Ok(vfs_error(e) as i32),
        };

        let filestat = Filestat::new(
            stats.inode,
            kind_to_filetype(&stats.kind),
            stats.size,
            stats.access.as_u64() * 1_000_000_000,
            stats.modification.as_u64() * 1_000_000_000,
            stats.creation.as_u64() * 1_000_000_000,
        );
        let memory = get_memory(&caller)?;
        let data = memory.data_mut(&mut caller);
        let off = buf_ptr as usize;
        data[off..off + filestat.as_bytes().len()].copy_from_slice(filestat.as_bytes());
        xila::log::information!("path_filestat_get: wrote {} bytes", filestat.as_bytes().len());
        Ok(0)
    }
}
