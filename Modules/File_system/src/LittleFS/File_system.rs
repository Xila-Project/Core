use core::mem::MaybeUninit;
use std::{collections::BTreeMap, ffi::CString, sync::RwLock};

use crate::{
    Device::Device_type, File_system_traits, Get_new_file_identifier, Local_file_identifier_type,
    Mode_type, Path_type, Size_type, Statistics_type,
};

use super::{littlefs, Configuration_type, Convert_result, Error_type, File_type, Result_type};

struct Inner_type {
    File_system: littlefs::lfs_t,
    Open_files: BTreeMap<Local_file_identifier_type, File_type>,
}

pub struct File_system_type {
    Inner: RwLock<Inner_type>,
    Cache_size: usize,
}

impl Drop for File_system_type {
    fn drop(&mut self) {
        // - Close all the open files
        let mut Inner = self.Inner.write().unwrap();

        let Keys = Inner.Open_files.keys().cloned().collect::<Vec<_>>();

        for Key in Keys {
            if let Some(File) = Inner.Open_files.remove(&Key) {
                let _ = File.Close(&mut Inner.File_system);
            }
        }

        let Configuration =
            unsafe { Box::from_raw(Inner.File_system.cfg as *mut littlefs::lfs_config) };

        let _Read_buffer = unsafe {
            Vec::from_raw_parts(
                Configuration.read_buffer as *mut u8,
                0,
                Configuration.cache_size as usize,
            )
        };
        let _Write_buffer = unsafe {
            Vec::from_raw_parts(
                Configuration.prog_buffer as *mut u8,
                0,
                Configuration.cache_size as usize,
            )
        };
        let _Look_ahead_buffer = unsafe {
            Vec::from_raw_parts(
                Configuration.lookahead_buffer as *mut u8,
                0,
                Configuration.lookahead_size as usize,
            )
        };

        // Get the device
        let _Device =
            unsafe { Box::from_raw(Inner.File_system.cfg.read().context as *mut Device_type) };

        // - Unmount the file system
        unsafe {
            littlefs::lfs_unmount(&mut Inner.File_system as *mut _);
        }

        // Configuration, Buffer sand Device are dropped here
    }
}

impl File_system_type {
    pub fn New(Device: Device_type, Cache_size: usize) -> Result_type<Self> {
        let Block_size = Device
            .Get_block_size()
            .map_err(|_| Error_type::Input_output)?;
        let Size = Device.Get_size().map_err(|_| Error_type::Input_output)?;

        let Configuration: littlefs::lfs_config = Configuration_type::New(
            Device,
            Block_size,
            usize::from(Size),
            Cache_size,
            Cache_size,
        )
        .ok_or(Error_type::Invalid_parameter)?
        .try_into()
        .map_err(|_| Error_type::Invalid_parameter)?;

        let Configuration = Box::new(Configuration);

        let mut File_system = MaybeUninit::<littlefs::lfs_t>::uninit();

        Convert_result(unsafe {
            littlefs::lfs_mount(
                File_system.as_mut_ptr() as *mut _,
                Box::into_raw(Configuration),
            )
        })?;

        let Inner = Inner_type {
            File_system: unsafe { File_system.assume_init() },
            Open_files: BTreeMap::new(),
        };

        Ok(Self {
            Inner: RwLock::new(Inner),
            Cache_size,
        })
    }

    pub fn Format(Device: Device_type, Cache_size: usize) -> Result_type<()> {
        let Block_size = Device
            .Get_block_size()
            .map_err(|_| Error_type::Input_output)?;
        let Size = Device.Get_size().map_err(|_| Error_type::Input_output)?;

        let Configuration: littlefs::lfs_config = Configuration_type::New(
            Device,
            Block_size,
            usize::from(Size),
            Cache_size,
            Cache_size,
        )
        .ok_or(Error_type::Invalid_parameter)?
        .try_into()
        .map_err(|_| Error_type::Invalid_parameter)?;

        let Configuration = Box::new(Configuration);

        let mut File_system = MaybeUninit::<littlefs::lfs_t>::uninit();

        Convert_result(unsafe {
            littlefs::lfs_format(File_system.as_mut_ptr(), Box::into_raw(Configuration))
        })?;

        Ok(())
    }

    fn Borrow_mutable_inner_2_splited(
        Inner_2: &mut Inner_type,
    ) -> (
        &mut littlefs::lfs_t,
        &mut BTreeMap<Local_file_identifier_type, File_type>,
    ) {
        (&mut Inner_2.File_system, &mut Inner_2.Open_files)
    }
}

unsafe impl Send for File_system_type {}

unsafe impl Sync for File_system_type {}

impl File_system_traits for File_system_type {
    fn Open(
        &self,
        Task: Task::Task_identifier_type,
        Path: &dyn AsRef<Path_type>,
        Flags: crate::Flags_type,
    ) -> crate::Result_type<Local_file_identifier_type> {
        let mut Inner = self.Inner.write()?;

        let File = File_type::Open(&mut Inner.File_system, Task, Path, Flags, self.Cache_size)?;

        let File_identifier = Get_new_file_identifier(Task, &Inner.Open_files)?;

        if Inner.Open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error.into());
        }

        Ok(File_identifier)
    }

    fn Close(&self, File: Local_file_identifier_type) -> crate::Result_type<()> {
        let mut Inner = self.Inner.write()?;

        let File = Inner
            .Open_files
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Close(&mut Inner.File_system)?;

        Ok(())
    }

    fn Close_all(&self, Task: Task::Task_identifier_type) -> crate::Result_type<()> {
        let mut Inner = self.Inner.write()?;

        // Get all the keys of the open files that belong to the task
        let Keys = Inner
            .Open_files
            .keys()
            .filter(|Key| Key.Split().0 == Task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the files corresponding to the keys
        for Key in Keys {
            if let Some(File) = Inner.Open_files.remove(&Key) {
                File.Close(&mut Inner.File_system)?;
            }
        }

        Ok(())
    }

    fn Duplicate_file_identifier(
        &self,
        File: Local_file_identifier_type,
    ) -> crate::Result_type<Local_file_identifier_type> {
        let (Task, _) = File.Split();

        let mut Inner = self.Inner.write()?;

        let File = Inner
            .Open_files
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        let File = File.clone();

        let File_identifier = Get_new_file_identifier(Task, &Inner.Open_files)?;

        if Inner.Open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error.into());
        }

        Ok(File_identifier)
    }

    fn Transfert_file_identifier(
        &self,
        New_task: Task::Task_identifier_type,
        File: Local_file_identifier_type,
    ) -> crate::Result_type<Local_file_identifier_type> {
        let mut Inner = self.Inner.write()?;

        let File = Inner
            .Open_files
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        let File_identifier = Get_new_file_identifier(New_task, &Inner.Open_files)?;

        if Inner.Open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error.into());
        }

        Ok(File_identifier)
    }

    fn Delete(&self, Path: &dyn AsRef<Path_type>) -> crate::Result_type<()> {
        let Path =
            CString::new(Path.as_ref().As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let mut Inner = self.Inner.write()?;

        Convert_result(unsafe {
            littlefs::lfs_remove(&mut Inner.File_system as *mut _, Path.as_ptr())
        })?;

        Ok(())
    }

    fn Read(
        &self,
        File: Local_file_identifier_type,
        Buffer: &mut [u8],
    ) -> crate::Result_type<Size_type> {
        let mut Inner = self.Inner.write()?;

        let (File_system, Open_files) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Read(File_system, Buffer)?)
    }

    fn Write(
        &self,
        File: Local_file_identifier_type,
        Buffer: &[u8],
    ) -> crate::Result_type<Size_type> {
        let mut Inner = self.Inner.write()?;

        let (File_system, Open_files) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Write(File_system, Buffer)?)
    }

    fn Move(
        &self,
        Source: &dyn AsRef<Path_type>,
        Destination: &dyn AsRef<Path_type>,
    ) -> crate::Result_type<()> {
        let Source =
            CString::new(Source.as_ref().As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let Destination = CString::new(Destination.as_ref().As_str())
            .map_err(|_| Error_type::Invalid_parameter)?;

        let mut Inner = self.Inner.write()?;

        Convert_result(unsafe {
            littlefs::lfs_rename(
                &mut Inner.File_system as *mut _,
                Source.as_ptr(),
                Destination.as_ptr(),
            )
        })?;

        Ok(())
    }

    fn Set_position(
        &self,
        File: Local_file_identifier_type,
        Position: &crate::Position_type,
    ) -> crate::Result_type<Size_type> {
        let mut Inner = self.Inner.write()?;

        let (File_system, Open_files) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Set_position(File_system, Position)?)
    }

    fn Flush(&self, File: Local_file_identifier_type) -> crate::Result_type<()> {
        let mut Inner = self.Inner.write()?;

        let (File_system, Open_files) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Flush(File_system)?)
    }

    fn Get_statistics(
        &self,
        File: Local_file_identifier_type,
    ) -> crate::Result_type<Statistics_type> {
        let mut Inner = self.Inner.write()?;

        let (File_system, Open_files) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Get_statistics(File_system)?)
    }

    fn Get_mode(&self, File: Local_file_identifier_type) -> crate::Result_type<Mode_type> {
        Ok(self
            .Inner
            .read()?
            .Open_files
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .Get_mode())
    }
}

#[cfg(test)]
mod Tests {

    use std::sync::Arc;

    use crate::{Device_trait, Position_type};

    use super::*;

    struct Mock_device_type(RwLock<(Vec<u8>, usize)>);

    impl Mock_device_type {
        const Block_size: usize = 512;

        pub fn New(Size: usize) -> Self {
            assert!(Size % Self::Block_size == 0);

            let mut Data: Vec<u8> = vec![];
            Data.resize(Size, 0);

            Self(RwLock::new((Data, 0)))
        }

        pub fn Get_block_count(&self) -> usize {
            self.0.read().unwrap().0.len() / Self::Block_size
        }
    }

    impl Device_trait for Mock_device_type {
        fn Read(&self, Buffer: &mut [u8]) -> crate::Result_type<Size_type> {
            let mut Inner = self
                .0
                .try_write()
                .map_err(|_| crate::Error_type::Ressource_busy)?;
            let (Data, Position) = &mut *Inner;

            let Read_size = Buffer.len().min(Data.len().saturating_sub(*Position));
            Buffer[..Read_size].copy_from_slice(&Data[*Position..*Position + Read_size]);
            *Position += Read_size;
            Ok(Read_size.into())
        }

        fn Write(&self, Buffer: &[u8]) -> crate::Result_type<Size_type> {
            let mut Inner = self
                .0
                .write()
                .map_err(|_| crate::Error_type::Ressource_busy)?;
            let (Data, Position) = &mut *Inner;

            Data[*Position..*Position + Buffer.len()].copy_from_slice(Buffer);
            *Position += Buffer.len();
            Ok(Buffer.len().into())
        }

        fn Get_size(&self) -> crate::Result_type<Size_type> {
            let Inner = self
                .0
                .read()
                .map_err(|_| crate::Error_type::Ressource_busy)?;
            Ok(Size_type::New(Inner.0.len() as u64))
        }

        fn Set_position(&self, Position: &Position_type) -> crate::Result_type<Size_type> {
            let mut Inner = self
                .0
                .write()
                .map_err(|_| crate::Error_type::Ressource_busy)?;
            let (Data, Device_position) = &mut *Inner;

            match Position {
                Position_type::Start(Position) => *Device_position = *Position as usize,
                Position_type::Current(Position) => {
                    *Device_position = (*Device_position as isize + *Position as isize) as usize
                }
                Position_type::End(Position) => {
                    *Device_position = (Data.len() as isize - *Position as isize) as usize
                }
            }

            Ok(Size_type::New(*Device_position as u64))
        }

        fn Erase(&self) -> crate::Result_type<()> {
            let mut Inner = self
                .0
                .write()
                .map_err(|_| crate::Error_type::Ressource_busy)?;

            let (Data, Position) = &mut *Inner;

            Data[*Position..*Position + Self::Block_size].fill(0);

            Ok(())
        }

        fn Flush(&self) -> crate::Result_type<()> {
            Ok(())
        }

        fn Get_block_size(&self) -> crate::Result_type<usize> {
            Ok(Self::Block_size)
        }
    }

    const Cache_size: usize = 256;

    fn Initialize() -> File_system_type {
        let Mock_device = Mock_device_type::New(2048 * 512);

        let Device = Device_type::New(Arc::new(Mock_device));

        File_system_type::Format(Device.clone(), Cache_size).unwrap();

        File_system_type::New(Device, Cache_size).unwrap()
    }

    #[test]
    fn Test_open_close_delete() {
        crate::Tests::Test_open_close_delete(Initialize());
    }

    #[test]
    fn Test_read_write() {
        crate::Tests::Test_read_write(Initialize());
    }

    #[test]
    fn Test_move() {
        crate::Tests::Test_move(Initialize());
    }

    #[test]
    fn Test_set_position() {
        crate::Tests::Test_set_position(Initialize());
    }

    #[test]
    fn Test_flush() {
        crate::Tests::Test_flush(Initialize());
    }

    #[test]
    fn Test_set_owner() {
        crate::Tests::Test_set_owner(Initialize());
    }

    #[test]
    fn Test_set_permissions() {
        crate::Tests::Test_set_permissions(Initialize());
    }
}
