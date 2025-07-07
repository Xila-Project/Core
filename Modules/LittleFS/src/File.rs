use core::{
    ffi::c_void,
    mem::{forget, MaybeUninit},
};

use alloc::{boxed::Box, ffi::CString, rc::Rc, vec, vec::Vec};
use File_system::{
    Error_type, File_system_identifier_type, Flags_type, Inode_type, Metadata_type, Mode_type,
    Path_type, Position_type, Result_type, Size_type, Statistics_type, Time_type, Type_type,
};
use Users::{Group_identifier_type, User_identifier_type};

use super::{littlefs, Convert_flags, Convert_result};

fn Convert_position(Position: &Position_type) -> (i32, i32) {
    match Position {
        Position_type::Start(Position) => (
            *Position as i32,
            littlefs::lfs_whence_flags_LFS_SEEK_SET as i32,
        ),
        Position_type::Current(Position) => (
            *Position as i32,
            littlefs::lfs_whence_flags_LFS_SEEK_CUR as i32,
        ),
        Position_type::End(Position) => (
            *Position as i32,
            littlefs::lfs_whence_flags_LFS_SEEK_END as i32,
        ),
    }
}

struct Inner_type {
    file: littlefs::lfs_file_t,
    flags: Flags_type,
    cache_size: usize,
}

impl Drop for Inner_type {
    fn drop(&mut self) {
        unsafe {
            let configuration = Box::from_raw(self.file.cfg as *mut littlefs::lfs_file_config);

            let Attributes = Box::from_raw(configuration.attrs);

            let _Metadata = Box::from_raw(Attributes.buffer as *mut Metadata_type);

            let _Buffer = Vec::from_raw_parts(configuration.buffer as *mut u8, 0, self.cache_size);
        }
    }
}

#[derive(Clone)]
pub struct File_type(Rc<Inner_type>);

impl File_type {
    pub fn open(
        file_system: &mut super::littlefs::lfs_t,
        path: &Path_type,
        flags: Flags_type,
        cache_size: usize,
        time: Time_type,
        user: User_identifier_type,
        group: Group_identifier_type,
    ) -> Result_type<Self> {
        let metadata = if flags.Get_open().Get_create() {
            Metadata_type::Get_default(Type_type::File, time, user, group)
                .ok_or(Error_type::Invalid_parameter)?
        } else {
            Self::Get_metadata_from_path(file_system, path)?
        };

        let Path = CString::new(path.As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let Little_fs_flags = Convert_flags(flags);

        let Metadata_buffer = Box::new(metadata);

        let Attribute = Box::new(littlefs::lfs_attr {
            type_: Metadata_type::IDENTIFIER,
            buffer: Box::into_raw(Metadata_buffer) as *mut c_void,
            size: size_of::<Metadata_type>() as u32,
        });

        let mut Buffer = vec![0_u8; cache_size];

        // - Create the configuration
        let Configuration = Box::new(littlefs::lfs_file_config {
            buffer: Buffer.as_mut_ptr() as *mut c_void,
            attrs: Box::into_raw(Attribute),
            attr_count: 1,
        });

        forget(Buffer); // Prevent the buffer from being deallocated

        let File = unsafe {
            let file = MaybeUninit::<littlefs::lfs_file_t>::uninit();

            let File = Self(Rc::new(Inner_type {
                file: file.assume_init(),
                flags,
                cache_size,
            }));

            Convert_result(littlefs::lfs_file_opencfg(
                file_system as *mut _,
                &File.0.file as *const _ as *mut _,
                Path.as_ptr(),
                Little_fs_flags,
                Box::into_raw(Configuration),
            ))?;

            File
        };

        // Ensure that metadata is written to created files
        if flags.Get_open().Get_create() {
            File.Flush(file_system)?;
        }

        Ok(File)
    }

    pub fn Close(self, File_system: &mut super::littlefs::lfs_t) -> Result_type<()> {
        unsafe {
            Convert_result(littlefs::lfs_file_close(
                File_system as *mut _,
                &self.0.file as *const _ as *mut _,
            ))?;
        }
        Ok(())
    }

    pub fn Read(
        &mut self,
        file_system: &mut super::littlefs::lfs_t,
        buffer: &mut [u8],
    ) -> Result_type<Size_type> {
        let bytes_read = unsafe {
            Convert_result(littlefs::lfs_file_read(
                file_system as *mut _,
                &self.0.file as *const _ as *mut _,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
            ))?
        };

        Ok(Size_type::from(bytes_read as usize))
    }

    pub fn Write(
        &mut self,
        file_system: &mut super::littlefs::lfs_t,
        buffer: &[u8],
    ) -> Result_type<Size_type> {
        let bytes_written = unsafe {
            Convert_result(littlefs::lfs_file_write(
                file_system as *mut _,
                &self.0.file as *const _ as *mut _,
                buffer.as_ptr() as *const _,
                buffer.len() as u32,
            ))?
        };

        Ok(Size_type::from(bytes_written as usize))
    }

    pub fn Set_position(
        &self,
        file_system: &mut super::littlefs::lfs_t,
        position: &Position_type,
    ) -> Result_type<Size_type> {
        let (offset, whence) = Convert_position(position);

        let Offset = unsafe {
            Convert_result(littlefs::lfs_file_seek(
                file_system as *mut _,
                &self.0.file as *const _ as *mut _,
                offset,
                whence,
            ))?
        };

        Ok(Size_type::from(Offset as usize))
    }

    pub fn Flush(&self, File_system: &mut super::littlefs::lfs_t) -> Result_type<()> {
        unsafe {
            Convert_result(littlefs::lfs_file_sync(
                File_system as *mut _,
                &self.0.file as *const _ as *mut _,
            ))?;
        }

        Ok(())
    }

    pub fn Get_statistics(
        &self,
        file_system: &mut super::littlefs::lfs_t,
    ) -> Result_type<Statistics_type> {
        let metadata = self.Get_metadata()?;

        let Size = self.Get_size(file_system)?;

        let Statistics = Statistics_type::new(
            File_system_identifier_type::New(0),
            Inode_type::New(0),
            1,
            Size,
            metadata.Get_creation_time(),
            metadata.Get_modification_time(),
            metadata.Get_access_time(),
            metadata.Get_type(),
            metadata.Get_permissions(),
            metadata.Get_user(),
            metadata.Get_group(),
        );

        Ok(Statistics)
    }

    pub fn Get_metadata(&self) -> Result_type<&Metadata_type> {
        let configuration = unsafe { self.0.file.cfg.read() };

        if configuration.attr_count == 0 {
            return Err(Error_type::No_attribute);
        }

        let Attributes = unsafe { configuration.attrs.read() };

        if Attributes.size != size_of::<Metadata_type>() as u32 {
            return Err(Error_type::No_attribute);
        }

        let Metadata = unsafe { &*(Attributes.buffer as *const Metadata_type) };

        Ok(Metadata)
    }

    pub fn Get_mode(&self) -> Mode_type {
        self.0.flags.Get_mode()
    }

    pub fn Get_size(&self, File_system: &mut super::littlefs::lfs_t) -> Result_type<Size_type> {
        let size = unsafe {
            Convert_result(littlefs::lfs_file_size(
                File_system as *mut _,
                &self.0.file as *const _ as *mut _,
            ))?
        };

        Ok(Size_type::from(size as usize))
    }

    pub fn Get_metadata_from_path(
        file_system: &mut super::littlefs::lfs_t,
        path: &Path_type,
    ) -> Result_type<Metadata_type> {
        let path = CString::new(path.As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let mut Metadata = MaybeUninit::<Metadata_type>::uninit();

        Convert_result(unsafe {
            littlefs::lfs_getattr(
                file_system as *mut _,
                path.as_ptr(),
                Metadata_type::IDENTIFIER,
                Metadata.as_mut_ptr() as *mut c_void,
                size_of::<Metadata_type>() as u32,
            )
        })?;

        Ok(unsafe { Metadata.assume_init() })
    }

    pub fn Set_metadata_from_path(
        file_system: &mut super::littlefs::lfs_t,
        path: &Path_type,
        metadata: &Metadata_type,
    ) -> Result_type<()> {
        let path = CString::new(path.As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        Convert_result(unsafe {
            littlefs::lfs_setattr(
                file_system as *mut _,
                path.as_ptr(),
                Metadata_type::IDENTIFIER,
                metadata as *const _ as *const c_void,
                size_of::<Metadata_type>() as u32,
            )
        })?;

        Ok(())
    }
}
