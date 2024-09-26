use core::{mem::MaybeUninit, pin::Pin};
use std::{collections::BTreeMap, ffi::CString, mem::transmute, sync::RwLock};

use crate::{File_system_traits, Path_type, Size_type, Unique_file_identifier_type};

use super::{
    littlefs, Callbacks, Configuration_type, Convert_error, Error_type, File_type, Result_type,
};

struct Inner_type<const Cache_size: usize> {
    Configuration: littlefs::lfs_config,
    Read_buffer: [u8; Cache_size],
    Write_buffer: [u8; Cache_size],
    Lookahead_buffer: [u8; Cache_size],
}

pub struct File_system_type<const Cache_size: usize> {
    Inner: Pin<Box<Inner_type<Cache_size>>>,
    File_system: littlefs::lfs_t,
    Openned_files: RwLock<BTreeMap<usize, RwLock<littlefs::lfs_file_t>>>,
}

impl<const Cache_size: usize> File_system_type<Cache_size> {
    pub fn New(
        Device_file: Unique_file_identifier_type,
        Configuration: Configuration_type,
    ) -> Result_type<Self> {
        let mut Inner = Box::pin(Inner_type {
            Configuration: unsafe { MaybeUninit::uninit().assume_init() },
            Read_buffer: [0; Cache_size],
            Write_buffer: [0; Cache_size],
            Lookahead_buffer: [0; Cache_size],
        });

        let mut Inner_reference = Inner.as_mut();

        let Configuration: littlefs::lfs_config = Configuration
            .Set_buffers(
                Inner_reference.Read_buffer.as_mut_ptr(),
                Inner_reference.Write_buffer.as_mut_ptr(),
                Inner_reference.Lookahead_buffer.as_mut_ptr(),
            )
            .Set_context(Device_file)
            .try_into()
            .map_err(|_| Error_type::Invalid_parameter)?;

        Inner_reference.Configuration = Configuration;

        let Configuration_pointer = &Inner_reference.Configuration as *const _;

        let mut File_system = unsafe { MaybeUninit::<littlefs::lfs_t>::uninit().assume_init() };

        Convert_error(unsafe {
            littlefs::lfs_mount(&mut File_system as *mut _, Configuration_pointer)
        })?;

        Ok(Self {
            Inner,
            File_system,
            Openned_files: RwLock::new(BTreeMap::new()),
        })
    }

    pub fn Format(
        Device_file: Unique_file_identifier_type,
        Configuration: Configuration_type,
    ) -> Result_type<()> {
        let mut Inner = Box::pin(Inner_type {
            Configuration: unsafe { MaybeUninit::uninit().assume_init() },
            Read_buffer: [0; Cache_size],
            Write_buffer: [0; Cache_size],
            Lookahead_buffer: [0; Cache_size],
        });

        let mut Inner_reference = Inner.as_mut();

        let Configuration: littlefs::lfs_config = Configuration
            .Set_buffers(
                Inner_reference.Read_buffer.as_mut_ptr(),
                Inner_reference.Write_buffer.as_mut_ptr(),
                Inner_reference.Lookahead_buffer.as_mut_ptr(),
            )
            .Set_context(Device_file)
            .try_into()
            .map_err(|_| Error_type::Invalid_parameter)?;

        Inner_reference.Configuration = Configuration;

        let Configuration_pointer = &Inner_reference.Configuration as *const _;

        let mut File_system = unsafe { MaybeUninit::<littlefs::lfs_t>::uninit().assume_init() };

        Convert_error(unsafe {
            littlefs::lfs_format(&mut File_system as *mut _, Configuration_pointer)
        })?;

        Ok(())
    }

    pub(crate) fn Get_pointer(&self) -> *mut littlefs::lfs_t {
        &self.File_system as *const _ as *mut _
    }
}

unsafe impl<const Buffer_size: usize> Send for File_system_type<Buffer_size> {}

unsafe impl<const Buffer_size: usize> Sync for File_system_type<Buffer_size> {}

impl<const Buffer_size: usize> File_system_traits for File_system_type<Buffer_size> {
    fn Open(
        &self,
        Task: Task::Task_identifier_type,
        Path: &dyn AsRef<crate::Path_type>,
        Flags: crate::Flags_type,
    ) -> crate::Result_type<crate::File_identifier_type> {
        let Path =
            CString::new(Path.as_ref().As_str()).map_err(|_| crate::Error_type::Invalid_input)?;

        let File = File_type::New();

        let File = Convert_error(unsafe {
            littlefs::lfs_file_open(
                &self.File_system as *const _ as *mut _,
                Path.as_ref().as_ptr(),
                Flags.bits(),
            )
        })?;

        Ok(File)
    }

    fn Close(
        &self,
        Task: Task::Task_identifier_type,
        File: crate::File_identifier_type,
    ) -> crate::Result_type<()> {
        todo!()
    }

    fn Close_all(&self, Task: Task::Task_identifier_type) -> crate::Result_type<()> {
        todo!()
    }

    fn Duplicate_file_identifier(
        &self,
        Task: Task::Task_identifier_type,
        File: crate::File_identifier_type,
    ) -> crate::Result_type<crate::File_identifier_type> {
        todo!()
    }

    fn Transfert_file_identifier(
        &self,
        Old_task: Task::Task_identifier_type,
        New_task: Task::Task_identifier_type,
        File: crate::File_identifier_type,
        New_file_identifier: Option<crate::File_identifier_type>,
    ) -> crate::Result_type<crate::File_identifier_type> {
        todo!()
    }

    fn Delete(&self, Path: &dyn AsRef<crate::Path_type>) -> crate::Result_type<()> {
        todo!()
    }

    fn Read(
        &self,
        Task: Task::Task_identifier_type,
        File: crate::File_identifier_type,
        Buffer: &mut [u8],
    ) -> crate::Result_type<crate::Size_type> {
        todo!()
    }

    fn Write(
        &self,
        Task: Task::Task_identifier_type,
        File: crate::File_identifier_type,
        Buffer: &[u8],
    ) -> crate::Result_type<crate::Size_type> {
        todo!()
    }

    fn Move(
        &self,
        Source: &dyn AsRef<crate::Path_type>,
        Destination: &dyn AsRef<crate::Path_type>,
    ) -> crate::Result_type<()> {
        todo!()
    }

    fn Set_position(
        &self,
        Task: Task::Task_identifier_type,
        File: crate::File_identifier_type,
        Position: &crate::Position_type,
    ) -> crate::Result_type<crate::Size_type> {
        todo!()
    }

    fn Flush(
        &self,
        Task: Task::Task_identifier_type,
        File: crate::File_identifier_type,
    ) -> crate::Result_type<()> {
        todo!()
    }

    fn Get_statistics(
        &self,
        Task: Task::Task_identifier_type,
        File: crate::File_identifier_type,
        File_system: crate::File_system_identifier_type,
    ) -> crate::Result_type<crate::Statistics_type> {
        todo!()
    }

    fn Get_mode(
        &self,
        Task: Task::Task_identifier_type,
        File: crate::File_identifier_type,
    ) -> crate::Result_type<crate::Mode_type> {
        todo!()
    }
}
