use std::{ffi::CString, mem::MaybeUninit, rc::Rc};

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

#[derive(Debug, Clone)]
pub struct File_type {
    Metadata: Metadata_type,
    Data: Rc<littlefs::lfs_file_t>,
    Flags: Flags_type,
}

impl File_type {
    pub fn Open(
        File_system: &mut super::littlefs::lfs_t,
        Task: Task_identifier_type,
        Path: &dyn AsRef<Path_type>,
        Flags: Flags_type,
    ) -> Result_type<Self> {
        let Path =
            CString::new(Path.as_ref().As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let Little_fs_flags = Convert_flags(Flags);

        let File = unsafe {
            let mut File = MaybeUninit::<littlefs::lfs_file_t>::uninit();

            Convert_result(littlefs::lfs_file_open(
                File_system as *mut _,
                File.as_mut_ptr(),
                Path.as_ptr(),
                Little_fs_flags,
            ))?;

            File
        };

        // - Check if the file metadata exists
        let Metadata = match Self::Get_metadata(File_system, &Path) {
            Ok(Metadata) => Metadata,
            Err(Error_type::No_attribute) => {
                // - Create the file metadata if it doesn't exist and the file should be created
                if Flags.Get_open().Get_create() || Flags.Get_open().Get_create_exclusive() {
                    let Metadata = Metadata_type::Get(Task, Flags)?;
                    Self::Set_metadata(File_system, &Path, &Metadata)?;
                    Metadata
                } else {
                    return Err(Error_type::No_attribute);
                }
            }
            Err(Error) => return Err(Error),
        };

        Ok(Self {
            Metadata,
            Data: Rc::new(unsafe { File.assume_init() }),
            Flags,
        })
    }

    pub fn Close(self, File_system: &mut super::littlefs::lfs_t) -> Result_type<()> {
        unsafe {
            Convert_result(littlefs::lfs_file_close(
                File_system as *mut _,
                &*self.Data as *const _ as *mut _,
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
                &*self.Data as *const _ as *mut _,
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
                &*self.Data as *const _ as *mut _,
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
                &*self.Data as *const _ as *mut _,
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
                &*self.Data as *const _ as *mut _,
            ))?;
        }

        Ok(())
    }

    pub fn Get_statistics(
        &self,
        File_system: &mut super::littlefs::lfs_t,
    ) -> Result_type<Statistics_type> {
        let Size = self.Get_size(File_system)?;

        let Statistics = Statistics_type::New(
            File_system_identifier_type::New(0),
            Inode_type::New(0),
            1,
            Size,
            self.Metadata.Access_time,
            self.Metadata.Modification_time,
            self.Metadata.Modification_time,
            self.Metadata.Type,
        );

        Ok(Statistics)
    }

    fn Set_metadata(
        File_system: &mut super::littlefs::lfs_t,
        Path: &CString,
        Metadata: &Metadata_type,
    ) -> Result_type<()> {
        let Metadata = Metadata.as_ref();

        unsafe {
            Convert_result(littlefs::lfs_setattr(
                File_system as *mut _,
                Path.as_ptr(),
                Metadata_type::Identifer,
                Metadata.as_ptr() as *const _,
                Metadata.len() as u32,
            ))?;
        }

        Ok(())
    }

    fn Get_metadata(
        File_system: &mut super::littlefs::lfs_t,
        Path: &CString,
    ) -> Result_type<Metadata_type> {
        let mut Metadata = unsafe { MaybeUninit::<Metadata_type>::uninit() };

        let Metadata_slice = unsafe {
            core::slice::from_raw_parts_mut(
                Metadata.as_mut_ptr() as *mut u8,
                core::mem::size_of::<Metadata_type>(),
            )
        };

        unsafe {
            Convert_result(littlefs::lfs_getattr(
                File_system as *mut _,
                Path.as_ptr(),
                Metadata_type::Identifer,
                Metadata_slice.as_mut_ptr() as *mut _,
                Metadata_slice.len() as u32,
            ))?;
        }

        Ok(unsafe { Metadata.assume_init() })
    }

    pub fn Get_mode(&self) -> Mode_type {
        self.Flags.Get_mode()
    }

    pub fn Get_size(&self, File_system: &mut super::littlefs::lfs_t) -> Result_type<Size_type> {
        let Size = unsafe {
            Convert_result(littlefs::lfs_file_size(
                File_system as *mut _,
                &*self.Data as *const _ as *mut _,
            ))?
        };

        Ok(Size_type::from(Size as usize))
    }
}
