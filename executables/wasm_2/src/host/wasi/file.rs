use alloc::vec::Vec;
use wasmi::Caller;
use xila::file_system::Position;

use crate::{
    define_wasi_module,
    host::{
        store::GlobalStore,
        translation::{
            TranslateFrom, TranslationSliceIterator, TranslationSliceStream, WasiVector,
            WasmPointer, WasmUsize, get_memory,
        },
        wasi::{context::FileSystemItem, error::vfs_error, types::Filestat},
    },
};

fn fd_type(ty: &FileSystemItem) -> u8 {
    match ty {
        FileSystemItem::File(_) => 4,
        FileSystemItem::Directory(_, _) => 3,
        FileSystemItem::CharacterDevice => 2,
        FileSystemItem::Stdout(_) => 2,
        FileSystemItem::Stderr(_) => 2,
    }
}

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

    fn fd_read(
        caller: Caller<GlobalStore>,
        fd: i32,
        iovs_ptr: i32,
        iovs_len: i32,
        nread_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        xila::log::information!(
            "fd_read: fd={}, iovs={:#x}, iovs_len={}, nread={:#x}",
            fd, iovs_ptr, iovs_len, nread_ptr
        );
        let memory = get_memory(&mut caller)?;


        let stream = WasiVector::new(iovs_ptr as _, iovs_len as _);
        let mut stream = unsafe { TranslationSliceStream::translate_from(stream, memory)? };


        while let Some(buf) = stream.next() {
            let buf: &mut [u8] = buf?;



        }


        let iovs: TranslationSliceIterator<
        let total_len: usize = iovs.iter().map(|i| i.1 as usize).sum();
        xila::log::information!("fd_read: total buffer length={}", total_len);
        let mut buf = alloc::vec![0u8; total_len];

        let nread = {
            let store = caller.data_mut();
            let entry = match store.wasi.files.iter_mut().find(|e| e.fd == fd) {
                Some(e) => e,
                None => return Ok(8), // Error::Badf
            };
            match &mut entry.ty {
                FileSystemItem::File(file) => match file.read(&mut buf) {
                    Ok(n) => n,
                    Err(e) => return Ok(vfs_error(e) as i32),
                },
                FileSystemItem::CharacterDevice => 0,
                _ => return Ok(8),
            }
        };
        xila::log::information!("fd_read: read {} bytes", nread);

        {
            let data = memory.data_mut(&mut caller);
            let mut offset = 0;
            for &(ptr, len) in &iovs {
                let n = core::cmp::min(len as usize, nread - offset);
                data[ptr as usize..ptr as usize + n].copy_from_slice(&buf[offset..offset + n]);
                offset += n;
            }
            write_i32(data, nread_ptr as usize, nread as i32);
        }

        Ok(0)
    }

    fn fd_write(
        caller: Caller<GlobalStore>,
        fd: i32,
        iovs_ptr: i32,
        iovs_len: i32,
        nwritten_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        xila::log::information!(
            "fd_write: fd={}, iovs={:#x}, iovs_len={}, nwritten={:#x}",
            fd, iovs_ptr, iovs_len, nwritten_ptr
        );
        let mut caller = caller;
        let memory = get_memory(&caller)?;

        let iovs: Vec<(u32, u32)> = {
            let data = memory.data(&caller);
            (0..iovs_len as usize).map(|i| {
                let off = (iovs_ptr + i as i32 * 8) as usize;
                (read_u32(data, off), read_u32(data, off + 4))
            }).collect()
        };

        let total_len: usize = iovs.iter().map(|i| i.1 as usize).sum();
        xila::log::information!("fd_write: total buffer length={}", total_len);
        let mut buf = alloc::vec![0u8; total_len];
        {
            let data = memory.data(&caller);
            let mut offset = 0;
            for &(ptr, len) in &iovs {
                let n = core::cmp::min(len as usize, buf.len() - offset);
                buf[offset..offset + n].copy_from_slice(&data[ptr as usize..ptr as usize + n]);
                offset += n;
            }
        }

        let nwritten = {
            let store = caller.data_mut();
            let entry = match store.wasi.files.iter_mut().find(|e| e.fd == fd) {
                Some(e) => e,
                None => return Ok(8),
            };
            match &mut entry.ty {
                FileSystemItem::File(file) => match file.write(&buf) {
                    Ok(n) => n,
                    Err(e) => return Ok(vfs_error(e) as i32),
                },
                FileSystemItem::CharacterDevice => buf.len(),
                FileSystemItem::Stdout(file) | FileSystemItem::Stderr(file) => match file.write(&buf) {
                    Ok(n) => n,
                    Err(e) => return Ok(vfs_error(e) as i32),
                },
                _ => return Ok(8),
            }
        };
        xila::log::information!("fd_write: wrote {} bytes", nwritten);

        {
            let data = memory.data_mut(&mut caller);
            write_i32(data, nwritten_ptr as usize, nwritten as i32);
        }

        Ok(0)
    }

    fn fd_close(
        caller: Caller<GlobalStore>,
        fd: i32,
    ) -> Result<i32, wasmi::Error> {
        let mut caller = caller;
        let store = caller.data_mut();
        let pos = store.wasi.files.iter().position(|e| e.fd == fd);
        match pos {
            Some(p) => { store.wasi.files.remove(p); Ok(0) }
            None => Ok(8),
        }
    }

    fn fd_seek(
        caller: Caller<GlobalStore>,
        fd: i32,
        offset: i64,
        whence: i32,
        newoffset_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        let mut caller = caller;
        let memory = get_memory(&caller)?;

        let new_offset = {
            let store = caller.data_mut();
            let entry = match store.wasi.files.iter_mut().find(|e| e.fd == fd) {
                Some(e) => e,
                None => return Ok(8),
            };
            match &mut entry.ty {
                FileSystemItem::File(file) => {
                    let pos = match whence {
                        0 => Position::Start(offset as u64),
                        1 => Position::Current(offset),
                        2 => Position::End(offset),
                        _ => return Ok(28),
                    };
                    match file.set_position(&pos) {
                        Ok(p) => p,
                        Err(e) => return Ok(vfs_error(e) as i32),
                    }
                }
                _ => return Ok(8),
            }
        };

        {
            let data = memory.data_mut(&mut caller);
            write_u64(data, newoffset_ptr as usize, new_offset);
        }

        Ok(0)
    }

    fn fd_fdstat_get(
        caller: Caller<GlobalStore>,
        fd: i32,
        stat_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        let mut caller = caller;
        let memory = get_memory(&caller)?;
        let (ftype, flags, rights, rights_inheriting) = {
            let store = caller.data();
            let entry = match store.wasi.files.iter().find(|e| e.fd == fd) {
                Some(e) => e,
                None => return Ok(8),
            };
            (fd_type(&entry.ty), entry.flags, entry.rights, entry.rights_inheriting)
        };

        let data = memory.data_mut(&mut caller);
        let off = stat_ptr as usize;
        data[off] = ftype;
        data[off + 1] = 0;
        data[off + 2..off + 4].copy_from_slice(&flags.to_le_bytes());
        data[off + 4..off + 12].copy_from_slice(&rights.to_le_bytes());
        data[off + 12..off + 20].copy_from_slice(&rights_inheriting.to_le_bytes());
        Ok(0)
    }

    fn fd_prestat_get(
        caller: Caller<GlobalStore>,
        fd: i32,
        prestat_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        let (name_len, has_prestat) = {
            let prestats = &caller.data().wasi.prestats;
            if fd >= 3 && (fd as usize - 3) < prestats.len() {
                (prestats[fd as usize - 3].name.len() as u32, true)
            } else {
                (0u32, false)
            }
        };
        if !has_prestat {
            return Ok(8);
        }
        let mut caller = caller;
        let memory = get_memory(&caller)?;
        let data = memory.data_mut(&mut caller);
        let off = prestat_ptr as usize;
        data[off] = 0;
        data[off + 1..off + 4].fill(0);
        data[off + 4..off + 8].copy_from_slice(&name_len.to_le_bytes());
        Ok(0)
    }

    fn fd_prestat_dir_name(
        caller: Caller<GlobalStore>,
        fd: i32,
        path_ptr: i32,
        path_len: i32,
    ) -> Result<i32, wasmi::Error> {
        let name = {
            let prestats = &caller.data().wasi.prestats;
            if fd >= 3 && (fd as usize - 3) < prestats.len() {
                Some(prestats[fd as usize - 3].name.clone())
            } else {
                None
            }
        };
        let name = match name {
            Some(n) => n,
            None => return Ok(8),
        };
        let mut caller = caller;
        let memory = get_memory(&caller)?;
        let data = memory.data_mut(&mut caller);
        let len = core::cmp::min(name.len(), path_len as usize);
        data[path_ptr as usize..path_ptr as usize + len].copy_from_slice(&name[..len]);
        Ok(0)
    }

    fn fd_filestat_get(
        caller: Caller<GlobalStore>,
        fd: i32,
        stat_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        let mut caller = caller;
        let memory = get_memory(&caller)?;

        let stats = {
            let store = caller.data_mut();
            let entry = match store.wasi.files.iter_mut().find(|e| e.fd == fd) {
                Some(e) => e,
                None => return Ok(8),
            };
            match &mut entry.ty {
                FileSystemItem::File(file) => match file.get_statistics() {
                    Ok(s) => s,
                    Err(e) => return Ok(vfs_error(e) as i32),
                },
                _ => return Ok(8),
            }
        };

        let filestat = Filestat::new(
            stats.inode,
            kind_to_filetype(&stats.kind),
            stats.size,
            stats.access.as_u64() * 1_000_000_000,
            stats.modification.as_u64() * 1_000_000_000,
            stats.creation.as_u64() * 1_000_000_000,
        );
        let data = memory.data_mut(&mut caller);
        let off = stat_ptr as usize;
        data[off..off + filestat.as_bytes().len()].copy_from_slice(filestat.as_bytes());
        Ok(0)
    }
}
