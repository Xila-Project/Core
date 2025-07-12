use core::{ffi::CStr, fmt::Debug, mem::MaybeUninit};

use alloc::{ffi::CString, rc::Rc, string::ToString};
use file_system::{Entry, Inode, Kind, Path, Result, Size};

use super::{convert_result, littlefs};

struct Inner {
    directory: littlefs::lfs_dir_t,
}

#[derive(Clone)]
pub struct Directory(Rc<Inner>);

impl Debug for Directory {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Directory_type")
            .field("Inner", &self.0.directory)
            .finish()
    }
}

impl Directory {
    pub fn create_directory(file_system: &mut super::littlefs::lfs_t, path: &Path) -> Result<()> {
        let path = CString::new(path.as_str()).unwrap();

        convert_result(unsafe { littlefs::lfs_mkdir(file_system as *mut _, path.as_ptr()) })?;

        Ok(())
    }

    pub fn open(file_system: &mut super::littlefs::lfs_t, path: &Path) -> Result<Self> {
        let path = CString::new(path.as_str()).unwrap();

        let directory = MaybeUninit::<littlefs::lfs_dir_t>::uninit();

        let directory = Self(Rc::new(Inner {
            directory: unsafe { directory.assume_init() },
        }));

        convert_result(unsafe {
            littlefs::lfs_dir_open(
                file_system as *mut _,
                &directory.0.directory as *const _ as *mut _,
                path.as_ptr(),
            )
        })?;

        Ok(directory)
    }

    pub fn rewind(&mut self, file_system: &mut super::littlefs::lfs_t) -> Result<()> {
        convert_result(unsafe {
            littlefs::lfs_dir_rewind(
                file_system as *mut _,
                &self.0.directory as *const _ as *mut _,
            )
        })?;

        Ok(())
    }

    pub fn get_position(&mut self, file_system: &mut super::littlefs::lfs_t) -> Result<Size> {
        let offset = convert_result(unsafe {
            littlefs::lfs_dir_tell(
                file_system as *mut _,
                &self.0.directory as *const _ as *mut _,
            )
        })?;

        Ok(Size::new(offset as u64))
    }

    pub fn set_position(
        &mut self,
        file_system: &mut littlefs::lfs_t,
        position: Size,
    ) -> Result<()> {
        convert_result(unsafe {
            littlefs::lfs_dir_seek(
                file_system as *const _ as *mut _,
                &self.0.directory as *const _ as *mut _,
                u64::from(position) as littlefs::lfs_off_t,
            )
        })?;

        Ok(())
    }

    pub fn read(&mut self, file_system: &mut super::littlefs::lfs_t) -> Result<Option<Entry>> {
        let informations = MaybeUninit::<littlefs::lfs_info>::uninit();

        let mut informations = unsafe { informations.assume_init() };

        let result = unsafe {
            littlefs::lfs_dir_read(
                file_system as *mut _,
                &self.0.directory as *const _ as *mut _,
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

        let entry = Entry::new(
            Inode::new(0),
            name,
            r#type,
            Size::new(informations.size as u64),
        );

        Ok(Some(entry))
    }

    pub fn close(&mut self, file_system: &mut super::littlefs::lfs_t) -> Result<()> {
        convert_result(unsafe {
            littlefs::lfs_dir_close(
                file_system as *mut _,
                &self.0.directory as *const _ as *mut _,
            )
        })?;

        Ok(())
    }
}
