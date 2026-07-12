use alloc::vec::Vec;
use wasmi::Caller;

use crate::{
    define_wasi_module,
    host::{
        store::GlobalStore,
        wasi::{context::FdType, error::vfs_error, memory::get_memory},
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

    fn fd_readdir(
        caller: Caller<GlobalStore>,
        fd: i32,
        buf_ptr: i32,
        buf_len: i32,
        cookie: i64,
        nread_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        xila::log::information!(
            "fd_readdir: fd={}, buffer={:#x}, length={}, cookie={}, nread={:#x}",
            fd, buf_ptr, buf_len, cookie, nread_ptr
        );
        let mut caller = caller;
        let entries = {
            let store = caller.data_mut();
            let entry = match store.wasi.fds.iter_mut().find(|e| e.fd == fd) {
                Some(e) => e,
                None => return Ok(8),
            };
            let dir = match &mut entry.ty {
                FdType::Directory(dir, _) => dir,
                _ => return Ok(8),
            };
            if let Err(e) = dir.rewind() {
                return Ok(vfs_error(e) as i32);
            }
            let mut entries: Vec<xila::file_system::Entry> = Vec::new();
            loop {
                match dir.read() {
                    Ok(Some(e)) => entries.push(e),
                    Ok(None) => break,
                    Err(e) => return Ok(vfs_error(e) as i32),
                }
            }
            entries
        };
        xila::log::information!("fd_readdir: {} entries", entries.len());

        let memory = get_memory(&caller)?;
        let data = memory.data_mut(&mut caller);

        let start = cookie as usize;
        let mut total_written = 0usize;

        for i in start..entries.len() {
            let remaining = buf_len as usize - total_written;
            if remaining < 24 {
                break;
            }
            let name_bytes = entries[i].name.as_bytes();
            let dirent_size = 24 + name_bytes.len();
            if dirent_size > remaining {
                break;
            }

            let off = buf_ptr as usize + total_written;
            let next_cookie = if i + 1 < entries.len() {
                (i + 1) as u64
            } else {
                u64::MAX
            };

            data[off..off + 8].copy_from_slice(&next_cookie.to_le_bytes());
            data[off + 8..off + 16].copy_from_slice(&entries[i].inode.to_le_bytes());
            data[off + 16..off + 20].copy_from_slice(&(name_bytes.len() as u32).to_le_bytes());
            data[off + 20] = kind_to_filetype(&entries[i].kind);
            data[off + 21..off + 24].fill(0);
            data[off + 24..off + 24 + name_bytes.len()].copy_from_slice(name_bytes);
            total_written += dirent_size;
        }

        data[nread_ptr as usize..nread_ptr as usize + 4].copy_from_slice(&(total_written as i32).to_le_bytes());
        xila::log::information!("fd_readdir: wrote {} bytes", total_written);
        Ok(0)
    }
}
