use std::{
    ffi::{CStr, CString},
    fmt::Debug,
    mem::MaybeUninit,
    rc::Rc,
};

use crate::{Entry_type, Inode_type, Path_type, Size_type, Type_type};

use super::{littlefs, Convert_result, Result_type};

struct Inner_type {
    Directory: littlefs::lfs_dir_t,
}

#[derive(Clone)]
pub struct Directory_type(Rc<Inner_type>);

impl Debug for Directory_type {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Formatter
            .debug_struct("Directory_type")
            .field("Inner", &self.0.Directory)
            .finish()
    }
}

impl Directory_type {
    pub fn Create_directory(
        File_system: &mut super::littlefs::lfs_t,
        Path: &Path_type,
    ) -> Result_type<()> {
        let Path = CString::new(Path.As_str()).unwrap();

        Convert_result(unsafe { littlefs::lfs_mkdir(File_system as *mut _, Path.as_ptr()) })?;

        Ok(())
    }

    pub fn Open(File_system: &mut super::littlefs::lfs_t, Path: &Path_type) -> Result_type<Self> {
        let Path = CString::new(Path.As_str()).unwrap();

        let Directory = MaybeUninit::<littlefs::lfs_dir_t>::uninit();

        let Directory = Self(Rc::new(Inner_type {
            Directory: unsafe { Directory.assume_init() },
        }));

        Convert_result(unsafe {
            littlefs::lfs_dir_open(
                File_system as *mut _,
                &Directory.0.Directory as *const _ as *mut _,
                Path.as_ptr(),
            )
        })?;

        Ok(Directory)
    }

    pub fn Rewind(&mut self, File_system: &mut super::littlefs::lfs_t) -> Result_type<()> {
        Convert_result(unsafe {
            littlefs::lfs_dir_rewind(
                File_system as *mut _,
                &self.0.Directory as *const _ as *mut _,
            )
        })?;

        Ok(())
    }

    pub fn Get_position(
        &mut self,
        File_system: &mut super::littlefs::lfs_t,
    ) -> Result_type<Size_type> {
        let Offset = Convert_result(unsafe {
            littlefs::lfs_dir_tell(
                File_system as *mut _,
                &self.0.Directory as *const _ as *mut _,
            )
        })?;

        Ok(Size_type::New(Offset as u64))
    }

    pub fn Set_position(
        &mut self,
        File_system: &mut littlefs::lfs_t,
        Position: Size_type,
    ) -> Result_type<()> {
        Convert_result(unsafe {
            littlefs::lfs_dir_seek(
                File_system as *const _ as *mut _,
                &self.0.Directory as *const _ as *mut _,
                u64::from(Position) as littlefs::lfs_off_t,
            )
        })?;

        Ok(())
    }

    pub fn Read(
        &mut self,
        File_system: &mut super::littlefs::lfs_t,
    ) -> Result_type<Option<Entry_type>> {
        let Informations = MaybeUninit::<littlefs::lfs_info>::uninit();

        let mut Informations = unsafe { Informations.assume_init() };

        let Result = unsafe {
            littlefs::lfs_dir_read(
                File_system as *mut _,
                &self.0.Directory as *const _ as *mut _,
                &mut Informations as *mut _,
            )
        };

        if Result == 0 {
            return Ok(None);
        }

        Convert_result(Result)?;

        let Name = unsafe { CStr::from_ptr(Informations.name.as_ptr()) };
        let Name = Name.to_str().unwrap().to_string();

        let Type = if Informations.type_ == littlefs::lfs_type_LFS_TYPE_DIR as u8 {
            Type_type::Directory
        } else {
            Type_type::File
        };

        let Entry = Entry_type::New(
            Inode_type::New(0),
            Name,
            Type,
            Size_type::New(Informations.size as u64),
        );

        Ok(Some(Entry))
    }

    pub fn Close(&mut self, File_system: &mut super::littlefs::lfs_t) -> Result_type<()> {
        Convert_result(unsafe {
            littlefs::lfs_dir_close(
                File_system as *mut _,
                &self.0.Directory as *const _ as *mut _,
            )
        })?;

        Ok(())
    }
}
