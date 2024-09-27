use std::{
    mem::MaybeUninit,
    sync::{Arc, RwLock},
};

use crate::{Flags_type, Path_type, Size_type};

use super::{littlefs, Convert_error, Convert_flags, Error_type, Result_type};

#[derive(Debug, Clone)]
pub struct File_type(Arc<RwLock<littlefs::lfs_file_t>>);

impl File_type {
    pub fn Open(
        File_system: &mut super::littlefs::lfs_t,
        Path: &dyn AsRef<Path_type>,
        Flags: Flags_type,
    ) -> Result_type<Self> {
        let Path = std::ffi::CString::new(Path.as_ref().As_str())
            .map_err(|_| Error_type::Invalid_parameter)?;

        let Flags = Convert_flags(Flags);

        unsafe {
            let mut File = MaybeUninit::<littlefs::lfs_file_t>::uninit().assume_init();

            let File_pointer = &mut File as *mut _;

            Convert_error(littlefs::lfs_file_open(
                File_system as *mut _,
                File_pointer,
                Path.as_ptr(),
                Flags,
            ))?;

            Ok(Self(Arc::new(RwLock::new(File))))
        }
    }

    pub fn Close(self, File_system: &mut super::littlefs::lfs_t) -> Result_type<()> {
        unsafe {
            Convert_error(littlefs::lfs_file_close(
                File_system as *mut _,
                &mut *self.0.write()? as *mut _,
            ))?;

            Ok(())
        }
    }

    pub fn Read(
        &self,
        File_system: &mut super::littlefs::lfs_t,
        Buffer: &mut [u8],
    ) -> Result_type<Size_type> {
        let mut Inner = self.0.write()?;

        let Bytes_read = unsafe {
            Convert_error(littlefs::lfs_file_read(
                File_system as *mut _,
                &mut *Inner as *mut _,
                Buffer.as_mut_ptr() as *mut _,
                Buffer.len() as u32,
            ))?
        };

        Ok(Size_type::from(Bytes_read as usize))
    }

    pub fn Write(
        &self,
        File_system: &mut super::littlefs::lfs_t,
        Buffer: &[u8],
    ) -> Result_type<Size_type> {
        let mut Inner = self.0.write()?;

        let Bytes_written = unsafe {
            Convert_error(littlefs::lfs_file_write(
                File_system as *mut _,
                &mut *Inner as *mut _,
                Buffer.as_ptr() as *const _,
                Buffer.len() as u32,
            ))?
        };

        Ok(Size_type::from(Bytes_written as usize))
    }
}
