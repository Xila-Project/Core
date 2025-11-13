use super::{convert_flags, convert_result, littlefs};
use crate::attributes::InternalAttributes;
use alloc::{boxed::Box, ffi::CString, vec, vec::Vec};
use core::{
    ffi::c_void,
    mem::{MaybeUninit, forget},
};
use file_system::{Attributes, Error, Flags, Path, Position, Result, Size};

fn convert_position(position: &Position) -> (i32, i32) {
    match position {
        Position::Start(position) => (
            *position as i32,
            littlefs::lfs_whence_flags_LFS_SEEK_SET as i32,
        ),
        Position::Current(position) => (
            *position as i32,
            littlefs::lfs_whence_flags_LFS_SEEK_CUR as i32,
        ),
        Position::End(position) => (
            *position as i32,
            littlefs::lfs_whence_flags_LFS_SEEK_END as i32,
        ),
    }
}

#[derive(Clone)]
pub struct File {
    cache_size: usize,
    file: littlefs::lfs_file_t,
}

impl File {
    pub fn lookup(
        file_system: &mut littlefs::lfs_t,
        path: &Path,
        flags: Flags,
        cache_size: usize,
    ) -> Result<Self> {
        let path = CString::new(path.as_str()).map_err(|_| Error::InvalidParameter)?;

        let little_fs_flags = convert_flags(flags);

        let mut buffer = vec![0_u8; cache_size];

        // - Create the configuration
        let configuration = Box::new(littlefs::lfs_file_config {
            buffer: buffer.as_mut_ptr() as *mut c_void,
            attrs: unsafe {
                InternalAttributes::new_uninitialized()
                    .assume_init()
                    .into_lfs_attributes()
            },
            attr_count: 1,
        });

        forget(buffer); // Prevent the buffer from being deallocated

        let file = unsafe {
            let file = MaybeUninit::<littlefs::lfs_file_t>::uninit();

            convert_result(littlefs::lfs_file_opencfg(
                file_system as *mut _,
                &file as *const _ as *mut _,
                path.as_ptr(),
                little_fs_flags,
                Box::into_raw(configuration),
            ))?;

            Self {
                cache_size,
                file: file.assume_init(),
            }
        };

        Ok(file)
    }

    pub fn create(file_system: &mut littlefs::lfs_t, path: &Path) -> Result<()> {
        let path = CString::new(path.as_str()).map_err(|_| Error::InvalidParameter)?;

        let mut file = MaybeUninit::<littlefs::lfs_file_t>::uninit();

        convert_result(unsafe {
            littlefs::lfs_file_open(
                file_system as *mut _,
                file.as_mut_ptr() as *mut _,
                path.as_ptr(),
                littlefs::lfs_open_flags_LFS_O_CREAT as i32
                    | littlefs::lfs_open_flags_LFS_O_EXCL as i32,
            )
        })?;

        convert_result(unsafe {
            littlefs::lfs_file_close(file_system as *mut _, file.as_mut_ptr() as *mut _)
        })?;

        Ok(())
    }

    pub fn get_attributes(&mut self, attributes: &mut Attributes) -> Result<()> {
        let internal_attributes =
            InternalAttributes::get_from_file_configuration(unsafe { &*self.file.cfg })
                .ok_or(Error::NoAttribute)?;

        internal_attributes.into_attributes(attributes)?;

        Ok(())
    }

    pub fn set_attributes(&mut self, attributes: &Attributes) -> Result<()> {
        let internal_attributes =
            InternalAttributes::get_mutable_from_file_configuration(unsafe { &*self.file.cfg })
                .ok_or(Error::NoAttribute)?;

        internal_attributes.from_attributes(attributes)?;

        Ok(())
    }

    pub fn close(mut self, file_system: &mut littlefs::lfs_t) -> Result<()> {
        unsafe {
            convert_result(littlefs::lfs_file_close(
                file_system as *mut _,
                &mut self.file as *mut _,
            ))?;

            let mut configuration = Box::from_raw(self.file.cfg as *mut littlefs::lfs_file_config);

            let _attributes = InternalAttributes::take_from_file_configuration(&mut *configuration);

            let _buffer = Vec::from_raw_parts(configuration.buffer as *mut u8, 0, self.cache_size);
        }

        Ok(())
    }

    pub fn read(
        &mut self,
        file_system: &mut littlefs::lfs_t,
        buffer: &mut [u8],
        absolue_position: Size,
    ) -> Result<usize> {
        let bytes_read = unsafe {
            convert_result(littlefs::lfs_file_seek(
                file_system as *mut _,
                &mut self.file as *mut _,
                absolue_position as i32,
                littlefs::lfs_whence_flags_LFS_SEEK_SET as i32,
            ))?;

            convert_result(littlefs::lfs_file_read(
                file_system as *mut _,
                &mut self.file as *mut _,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
            ))?
        };

        Ok(bytes_read as _)
    }

    pub fn write(
        &mut self,
        file_system: &mut littlefs::lfs_t,
        buffer: &[u8],
        absolue_position: Size,
    ) -> Result<usize> {
        let bytes_written = unsafe {
            convert_result(littlefs::lfs_file_seek(
                file_system as *mut _,
                &mut self.file as *mut _,
                absolue_position as i32,
                littlefs::lfs_whence_flags_LFS_SEEK_SET as i32,
            ))?;

            convert_result(littlefs::lfs_file_write(
                file_system as *mut _,
                &mut self.file as *mut _,
                buffer.as_ptr() as *const _,
                buffer.len() as u32,
            ))?
        };

        Ok(bytes_written as _)
    }

    pub fn set_position(
        &mut self,
        file_system: &mut littlefs::lfs_t,
        position: &Position,
    ) -> Result<Size> {
        let (offset, whence) = convert_position(position);

        let offset = unsafe {
            convert_result(littlefs::lfs_file_seek(
                file_system as *mut _,
                &mut self.file as *mut _,
                offset,
                whence,
            ))?
        };

        Ok(offset as _)
    }

    pub fn flush(&mut self, file_system: &mut littlefs::lfs_t) -> Result<()> {
        unsafe {
            convert_result(littlefs::lfs_file_sync(
                file_system as *mut _,
                &mut self.file as *mut _,
            ))?;
        }

        Ok(())
    }

    pub fn get_size(&mut self, file_system: &mut littlefs::lfs_t) -> Result<Size> {
        let size = unsafe {
            convert_result(littlefs::lfs_file_size(
                file_system as *mut _,
                &mut self.file as *mut _,
            ))?
        };

        Ok(size as _)
    }
}
