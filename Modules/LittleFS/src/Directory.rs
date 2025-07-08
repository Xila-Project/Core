use core::{ffi::CStr, fmt::Debug, mem::MaybeUninit};

use alloc::{ffi::CString, rc::Rc, string::ToString};
use File_system::{Entry_type, Inode_type, Path_type, Result_type, Size_type, Type_type};

use super::{littlefs, Convert_result};

struct Inner_type {
    directory: littlefs::lfs_dir_t,
}

#[derive(Clone)]
pub struct Directory_type(Rc<Inner_type>);

impl Debug for Directory_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Directory_type")
            .field("Inner", &self.0.directory)
            .finish()
    }
}

impl Directory_type {
    pub fn create_directory(
        file_system: &mut super::littlefs::lfs_t,
        path: &Path_type,
    ) -> Result_type<()> {
        let path = CString::new(path.As_str()).unwrap();

        Convert_result(unsafe { littlefs::lfs_mkdir(file_system as *mut _, path.as_ptr()) })?;

        Ok(())
    }

    pub fn Open(File_system: &mut super::littlefs::lfs_t, Path: &Path_type) -> Result_type<Self> {
        let path = CString::new(Path.As_str()).unwrap();

        let Directory = MaybeUninit::<littlefs::lfs_dir_t>::uninit();

        let Directory = Self(Rc::new(Inner_type {
            directory: unsafe { Directory.assume_init() },
        }));

        Convert_result(unsafe {
            littlefs::lfs_dir_open(
                File_system as *mut _,
                &Directory.0.directory as *const _ as *mut _,
                path.as_ptr(),
            )
        })?;

        Ok(Directory)
    }

    pub fn Rewind(&mut self, File_system: &mut super::littlefs::lfs_t) -> Result_type<()> {
        Convert_result(unsafe {
            littlefs::lfs_dir_rewind(
                File_system as *mut _,
                &self.0.directory as *const _ as *mut _,
            )
        })?;

        Ok(())
    }

    pub fn get_position(
        &mut self,
        file_system: &mut super::littlefs::lfs_t,
    ) -> Result_type<Size_type> {
        let offset = Convert_result(unsafe {
            littlefs::lfs_dir_tell(
                file_system as *mut _,
                &self.0.directory as *const _ as *mut _,
            )
        })?;

        Ok(Size_type::New(offset as u64))
    }

    pub fn Set_position(
        &mut self,
        file_system: &mut littlefs::lfs_t,
        position: Size_type,
    ) -> Result_type<()> {
        Convert_result(unsafe {
            littlefs::lfs_dir_seek(
                file_system as *const _ as *mut _,
                &self.0.directory as *const _ as *mut _,
                u64::from(position) as littlefs::lfs_off_t,
            )
        })?;

        Ok(())
    }

    pub fn Read(
        &mut self,
        file_system: &mut super::littlefs::lfs_t,
    ) -> Result_type<Option<Entry_type>> {
        let informations = MaybeUninit::<littlefs::lfs_info>::uninit();

        let mut Informations = unsafe { informations.assume_init() };

        let Result = unsafe {
            littlefs::lfs_dir_read(
                file_system as *mut _,
                &self.0.directory as *const _ as *mut _,
                &mut Informations as *mut _,
            )
        };

        if Result == 0 {
            return Ok(None);
        }

        Convert_result(Result)?;

        let Name = unsafe { CStr::from_ptr(Informations.name.as_ptr()) };
        let name = Name.to_str().unwrap().to_string();

        let Type = if Informations.type_ == littlefs::lfs_type_LFS_TYPE_DIR as u8 {
            Type_type::Directory
        } else {
            Type_type::File
        };

        let Entry = Entry_type::New(
            Inode_type::New(0),
            name,
            Type,
            Size_type::New(Informations.size as u64),
        );

        Ok(Some(Entry))
    }

    pub fn Close(&mut self, File_system: &mut super::littlefs::lfs_t) -> Result_type<()> {
        Convert_result(unsafe {
            littlefs::lfs_dir_close(
                File_system as *mut _,
                &self.0.directory as *const _ as *mut _,
            )
        })?;

        Ok(())
    }
}
