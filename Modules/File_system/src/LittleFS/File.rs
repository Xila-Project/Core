use std::{
    ffi::{c_void, CString},
    mem::{forget, MaybeUninit},
    ptr::null_mut,
    rc::Rc,
};

use littlefs2_sys::lfs_attr;
use Task::Task_identifier_type;

use crate::{
    File_system_identifier_type, Flags_type, Inode_type, Mode_type, Path_type, Position_type,
    Size_type, Statistics_type,
};

use super::{littlefs, Convert_flags, Convert_result, Error_type, Metadata_type, Result_type};

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
    File: littlefs::lfs_file_t,
    Flags: Flags_type,
    Cache_size: usize,
}

impl Drop for Inner_type {
    fn drop(&mut self) {
        unsafe {
            let Configuration = Box::from_raw(self.File.cfg as *mut littlefs::lfs_file_config);

            let Attributes = Box::from_raw(Configuration.attrs);

            let _Metadata = Box::from_raw(Attributes.buffer as *mut Metadata_type);

            let _Buffer = Vec::from_raw_parts(Configuration.buffer as *mut u8, 0, self.Cache_size);
        }
    }
}

#[derive(Clone)]
pub struct File_type(Rc<Inner_type>);

impl File_type {
    pub fn Open(
        File_system: &mut super::littlefs::lfs_t,
        Task: Task_identifier_type,
        Path: &dyn AsRef<Path_type>,
        Flags: Flags_type,
        Cache_size: usize,
    ) -> Result_type<Self> {
        let Path =
            CString::new(Path.as_ref().As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let Little_fs_flags = Convert_flags(Flags);

        // - Create the attribute
        let Metadata_buffer = Box::new(MaybeUninit::<Metadata_type>::uninit());

        let Attribute = Box::new(littlefs::lfs_attr {
            type_: Metadata_type::Identifier,
            buffer: Box::into_raw(Metadata_buffer) as *mut c_void,
            size: core::mem::size_of::<Metadata_type>() as u32,
        });

        let mut Buffer = vec![0_u8; Cache_size];

        // - Create the configuration
        let Configuration = Box::new(littlefs::lfs_file_config {
            buffer: Buffer.as_mut_ptr() as *mut c_void,
            attrs: Box::into_raw(Attribute),
            attr_count: 1,
        });

        forget(Buffer); // Prevent the buffer from being deallocated

        let File = unsafe {
            let File = MaybeUninit::<littlefs::lfs_file_t>::uninit();

            let File = Self(Rc::new(Inner_type {
                File: File.assume_init(),
                Flags,
                Cache_size,
            }));

            Convert_result(littlefs::lfs_file_opencfg(
                File_system as *mut _,
                &File.0.File as *const _ as *mut _,
                Path.as_ptr(),
                Little_fs_flags,
                Box::into_raw(Configuration),
            ))?;

            File
        };

        //      if Flags.Get_open().Get_create() {
        //          *File.Get_metadata_mutable()? = Metadata_type::Get(Task, Flags)?;
        //      }

        Ok(File)
    }

    pub fn Close(self, File_system: &mut super::littlefs::lfs_t) -> Result_type<()> {
        unsafe {
            Convert_result(littlefs::lfs_file_close(
                File_system as *mut _,
                &self.0.File as *const _ as *mut _,
            ))?;
        }
        Ok(())
    }

    pub fn Read(
        &mut self,
        File_system: &mut super::littlefs::lfs_t,
        Buffer: &mut [u8],
    ) -> Result_type<Size_type> {
        let Bytes_read = unsafe {
            Convert_result(littlefs::lfs_file_read(
                File_system as *mut _,
                &self.0.File as *const _ as *mut _,
                Buffer.as_mut_ptr() as *mut _,
                Buffer.len() as u32,
            ))?
        };

        Ok(Size_type::from(Bytes_read as usize))
    }

    pub fn Write(
        &mut self,
        File_system: &mut super::littlefs::lfs_t,
        Buffer: &[u8],
    ) -> Result_type<Size_type> {
        let Bytes_written = unsafe {
            Convert_result(littlefs::lfs_file_write(
                File_system as *mut _,
                &self.0.File as *const _ as *mut _,
                Buffer.as_ptr() as *const _,
                Buffer.len() as u32,
            ))?
        };

        Ok(Size_type::from(Bytes_written as usize))
    }

    pub fn Set_position(
        &self,
        File_system: &mut super::littlefs::lfs_t,
        Position: &Position_type,
    ) -> Result_type<Size_type> {
        let (Offset, Whence) = Convert_position(Position);

        let Offset = unsafe {
            Convert_result(littlefs::lfs_file_seek(
                File_system as *mut _,
                &self.0.File as *const _ as *mut _,
                Offset,
                Whence,
            ))?
        };

        Ok(Size_type::from(Offset as usize))
    }

    pub fn Flush(&self, File_system: &mut super::littlefs::lfs_t) -> Result_type<()> {
        unsafe {
            Convert_result(littlefs::lfs_file_sync(
                File_system as *mut _,
                &self.0.File as *const _ as *mut _,
            ))?;
        }

        Ok(())
    }

    pub fn Get_statistics(
        &self,
        File_system: &mut super::littlefs::lfs_t,
    ) -> Result_type<Statistics_type> {
        let Metadata = self.Get_metadata()?;

        let Size = self.Get_size(File_system)?;

        let Statistics = Statistics_type::New(
            File_system_identifier_type::New(0),
            Inode_type::New(0),
            1,
            Size,
            Metadata.Creation_time,
            Metadata.Modification_time,
            Metadata.Access_time,
            Metadata.Type,
        );

        Ok(Statistics)
    }

    fn Get_metadata(&self) -> Result_type<&Metadata_type> {
        let Configuration = unsafe { self.0.File.cfg.read() };

        if Configuration.attr_count == 0 {
            return Err(Error_type::No_attribute);
        }

        let Attributes = unsafe { Configuration.attrs.read() };

        if Attributes.size != size_of::<Metadata_type>() as u32 {
            return Err(Error_type::No_attribute);
        }

        let Metadata = unsafe { &*(Attributes.buffer as *const Metadata_type) };

        Ok(Metadata)
    }

    fn Get_metadata_mutable(&mut self) -> Result_type<&mut Metadata_type> {
        let Configuration = unsafe { self.0.File.cfg.read() };

        if Configuration.attr_count == 0 {
            return Err(Error_type::No_attribute);
        }

        let Attributes = unsafe { Configuration.attrs.read() };

        if Attributes.size != size_of::<Metadata_type>() as u32 {
            return Err(Error_type::No_attribute);
        }

        let Metadata = unsafe { &mut *(Attributes.buffer as *mut Metadata_type) };

        Ok(Metadata)
    }

    pub fn Get_mode(&self) -> Mode_type {
        self.0.Flags.Get_mode()
    }

    pub fn Get_size(&self, File_system: &mut super::littlefs::lfs_t) -> Result_type<Size_type> {
        let Size = unsafe {
            Convert_result(littlefs::lfs_file_size(
                File_system as *mut _,
                &self.0.File as *const _ as *mut _,
            ))?
        };

        Ok(Size_type::from(Size as usize))
    }
}
