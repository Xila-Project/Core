use std::mem::MaybeUninit;

use crate::Flags_type;

use super::{littlefs, Convert_error, Convert_flags, Error_type, Result_type};

pub struct File_type(RwLock<littlefs::lfs_file_t>);

impl File_type {
    pub fn Open<const Buffer_size: usize>(
        File_system: &mut super::File_system_type<Buffer_size>,
        Path: impl AsRef<str>,
        Flags: Flags_type,
    ) -> Result_type<Self> {
        let Path =
            std::ffi::CString::new(Path.as_ref()).map_err(|_| Error_type::Invalid_parameter)?;

        let Flags = Convert_flags(Flags);

        unsafe {
            let mut File = MaybeUninit::<littlefs::lfs_file_t>::uninit().assume_init();

            let File_pointer = &mut File as *mut _;

            Convert_error(littlefs::lfs_file_open(
                File_system.Get_pointer(),
                File_pointer,
                Path.as_ptr(),
                Flags,
            ))?;

            Ok(Self(File))
        }
    }

    pub fn Read<const Buffer_size: usize>(
        &mut self,
        File_system: &mut super::File_system_type<Buffer_size>,
        Buffer: &mut [u8],
    ) -> Result_type<usize> {
        unsafe {
            let mut Bytes_read = 0;

            Convert_error(littlefs::lfs_file_read(
                File_system.Get_pointer(),
                &self.0 as *const _ as *mut _,
                Buffer.as_mut_ptr() as *mut _,
                Buffer.len() as u32,
            ))?;

            Ok(Bytes_read)
        }
    }
}
