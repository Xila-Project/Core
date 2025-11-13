use core::{ffi::CStr, mem::MaybeUninit};

use crate::{error::convert_result, littlefs};
use alloc::{ffi::CString, string::ToString};
use file_system::{Attributes, Entry, Kind, Path, Result, Size};

#[derive(Clone)]
#[repr(transparent)]
pub struct Directory {
    directory: littlefs::lfs_dir_t,
}

impl Directory {
    pub fn lookup(file_system: &mut littlefs::lfs_t, path: &Path) -> Result<Self> {
        let path = CString::new(path.as_str()).unwrap();

        let mut directory = MaybeUninit::<littlefs::lfs_dir_t>::uninit();

        convert_result(unsafe {
            littlefs::lfs_dir_open(file_system as *mut _, directory.as_mut_ptr(), path.as_ptr())
        })?;

        Ok(Self {
            directory: unsafe { directory.assume_init() },
        })
    }

    pub fn get_attributes(&self, attributes: &mut Attributes) -> Result<()> {
        if let Some(kind) = attributes.get_mutable_kind() {
            *kind = file_system::Kind::Directory;
        }

        Ok(())
    }

    pub fn set_attributes(&self, _attributes: &Attributes) -> Result<()> {
        Ok(())
    }

    pub fn rewind(&mut self, file_system: &mut littlefs::lfs_t) -> Result<()> {
        convert_result(unsafe {
            littlefs::lfs_dir_rewind(file_system as *mut _, &mut self.directory as *mut _)
        })?;

        Ok(())
    }

    pub fn get_position(&mut self, file_system: &mut littlefs::lfs_t) -> Result<Size> {
        let offset = convert_result(unsafe {
            littlefs::lfs_dir_tell(file_system as *mut _, &mut self.directory as *mut _)
        })?;

        Ok(offset as _)
    }

    pub fn set_position(
        &mut self,
        file_system: &mut littlefs::lfs_t,
        position: Size,
    ) -> Result<()> {
        convert_result(unsafe {
            littlefs::lfs_dir_seek(
                file_system as *mut _,
                &mut self.directory as *mut _,
                position as littlefs::lfs_off_t,
            )
        })?;

        Ok(())
    }

    pub fn read(&mut self, file_system: &mut littlefs::lfs_t) -> Result<Option<Entry>> {
        let informations = MaybeUninit::<littlefs::lfs_info>::uninit();

        let mut informations = unsafe { informations.assume_init() };

        let result = unsafe {
            littlefs::lfs_dir_read(
                file_system as *mut _,
                &mut self.directory as *mut _,
                &mut informations as *mut _,
            )
        };

        if result == 0 {
            return Ok(None);
        }

        convert_result(result)?;

        let name = unsafe { CStr::from_ptr(informations.name.as_ptr()) };
        let name = name.to_str().unwrap().to_string();

        let r#type = if informations.type_ == littlefs::lfs_type_LFS_TYPE_DIR as u8 {
            Kind::Directory
        } else {
            Kind::File
        };

        let entry = Entry::new(0, name, r#type, informations.size as _);

        Ok(Some(entry))
    }

    pub fn close(mut self, file_system: &mut littlefs::lfs_t) -> Result<()> {
        convert_result(unsafe {
            littlefs::lfs_dir_close(file_system as *mut _, &mut self.directory as *mut _)
        })?;

        Ok(())
    }
}
