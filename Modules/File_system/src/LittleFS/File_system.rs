use core::mem::MaybeUninit;
use std::{collections::BTreeMap, ffi::CString, sync::RwLock};

use crate::{
    Device::Device_type, File_system_traits, Get_new_file_identifier, Local_file_identifier_type,
    Mode_type, Path_type, Size_type, Statistics_type,
};

use super::{littlefs, Configuration_type, Convert_result, Error_type, File_type, Result_type};

struct Inner_type<const Cache_size: usize> {
    Configuration: littlefs::lfs_config,
    Read_buffer: [u8; Cache_size],
    Write_buffer: [u8; Cache_size],
    Lookahead_buffer: [u8; Cache_size],
}

struct Inner_2_type {
    File_system: littlefs::lfs_t,
    Open_files: BTreeMap<Local_file_identifier_type, File_type>,
}

pub struct File_system_type<const Cache_size: usize> {
    _Inner: Box<Inner_type<Cache_size>>,
    Inner_2: RwLock<Inner_2_type>,
}

impl<const Cache_size: usize> Drop for File_system_type<Cache_size> {
    fn drop(&mut self) {
        // - Close all the open files
        let mut Inner = self.Inner_2.write().unwrap();

        let Keys = Inner.Open_files.keys().cloned().collect::<Vec<_>>();

        for Key in Keys {
            if let Some(File) = Inner.Open_files.remove(&Key) {
                let _ = File.Close(&mut Inner.File_system);
            }
        }

        // Get the device
        let _Device =
            unsafe { Box::from_raw(Inner.File_system.cfg.read().context as *mut Device_type) };

        // - Unmount the file system
        unsafe {
            littlefs::lfs_unmount(&mut Inner.File_system as *mut _);
        }

        // Device is dropped here
    }
}

impl<const Cache_size: usize> File_system_type<Cache_size> {
    pub fn New(Device: Device_type, Configuration: Configuration_type) -> Result_type<Self> {
        let mut _Inner = Box::new(Inner_type {
            Configuration: Configuration_type::Default_raw,
            Read_buffer: [0; Cache_size],
            Write_buffer: [0; Cache_size],
            Lookahead_buffer: [0; Cache_size],
        });

        let Inner_reference = _Inner.as_mut();

        let Configuration: littlefs::lfs_config = Configuration
            .Set_buffers(
                Inner_reference.Read_buffer.as_mut_ptr(),
                Inner_reference.Write_buffer.as_mut_ptr(),
                Inner_reference.Lookahead_buffer.as_mut_ptr(),
            )
            .Set_block_size(
                Device
                    .Get_block_size()
                    .map_err(|_| Error_type::Invalid_parameter)?,
            )
            .Set_context(Device)
            .try_into()
            .map_err(|_| Error_type::Invalid_parameter)?;

        Inner_reference.Configuration = Configuration;

        let Configuration_pointer = &Inner_reference.Configuration as *const _;

        let mut File_system = MaybeUninit::<littlefs::lfs_t>::uninit();

        Convert_result(unsafe {
            littlefs::lfs_mount(File_system.as_mut_ptr() as *mut _, Configuration_pointer)
        })?;

        let Inner_2 = Inner_2_type {
            File_system: unsafe { File_system.assume_init() },
            Open_files: BTreeMap::new(),
        };

        Ok(Self {
            _Inner,
            Inner_2: RwLock::new(Inner_2),
        })
    }

    pub fn Format(Device: Device_type, Configuration: Configuration_type) -> Result_type<()> {
        let mut Inner = Box::pin(Inner_type {
            Configuration: Configuration_type::Default_raw,
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
            .Set_context(Device)
            .try_into()
            .map_err(|_| Error_type::Invalid_parameter)?;

        Inner_reference.Configuration = Configuration;

        let Configuration_pointer = &Inner_reference.Configuration as *const _;

        let mut File_system = MaybeUninit::<littlefs::lfs_t>::uninit();

        Convert_result(unsafe {
            littlefs::lfs_format(File_system.as_mut_ptr(), Configuration_pointer)
        })?;

        Ok(())
    }

    fn Borrow_mutable_inner_2_splited(
        Inner_2: &mut Inner_2_type,
    ) -> (
        &mut littlefs::lfs_t,
        &mut BTreeMap<Local_file_identifier_type, File_type>,
    ) {
        (&mut Inner_2.File_system, &mut Inner_2.Open_files)
    }
}

unsafe impl<const Buffer_size: usize> Send for File_system_type<Buffer_size> {}

unsafe impl<const Buffer_size: usize> Sync for File_system_type<Buffer_size> {}

impl<const Buffer_size: usize> File_system_traits for File_system_type<Buffer_size> {
    fn Open(
        &self,
        Task: Task::Task_identifier_type,
        Path: &dyn AsRef<Path_type>,
        Flags: crate::Flags_type,
    ) -> crate::Result_type<Local_file_identifier_type> {
        let mut Inner = self.Inner_2.write()?;

        let File = File_type::Open(&mut Inner.File_system, Task, Path, Flags)?;

        let File_identifier = Get_new_file_identifier(Task, &Inner.Open_files)?;

        if Inner.Open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error.into());
        }

        Ok(File_identifier)
    }

    fn Close(&self, File: Local_file_identifier_type) -> crate::Result_type<()> {
        let mut Inner = self.Inner_2.write()?;

        let File = Inner
            .Open_files
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Close(&mut Inner.File_system)?;

        Ok(())
    }

    fn Close_all(&self, Task: Task::Task_identifier_type) -> crate::Result_type<()> {
        let mut Inner = self.Inner_2.write()?;

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

        let mut Inner = self.Inner_2.write()?;

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
        let mut Inner = self.Inner_2.write()?;

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

        let mut Inner = self.Inner_2.write()?;

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
        let mut Inner = self.Inner_2.write()?;

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
        let mut Inner = self.Inner_2.write()?;

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

        let mut Inner = self.Inner_2.write()?;

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
        let mut Inner = self.Inner_2.write()?;

        let (File_system, Open_files) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Set_position(File_system, Position)?)
    }

    fn Flush(&self, File: Local_file_identifier_type) -> crate::Result_type<()> {
        let mut Inner = self.Inner_2.write()?;

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
        let mut Inner = self.Inner_2.write()?;

        let (File_system, Open_files) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Get_statistics(File_system)?)
    }

    fn Get_mode(&self, File: Local_file_identifier_type) -> crate::Result_type<Mode_type> {
        Ok(self
            .Inner_2
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

    struct Mock_device_type<const Size: usize>(RwLock<(Box<[u8; Size]>, usize)>);

    impl<const Size: usize> Mock_device_type<Size> {
        const Block_size: usize = 512;

        pub fn New() -> Self {
            Self(RwLock::new((Box::new([0; Size]), 0)))
        }

        pub const fn Get_block_count(&self) -> usize {
            Size / Self::Block_size
        }
    }

    impl<const Size: usize> Device_trait for Mock_device_type<Size> {
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

    const Cache_size : usize = 512;

    fn Initialize() -> File_system_type<Cache_size> {
        let Mock_device = Mock_device_type::<204_800_000>::New();

        let Configuration =
            Configuration_type::default().Set_block_count(Mock_device.Get_block_count());

        let Device = Device_type::New(Arc::new(Mock_device));

        File_system_type::<Cache_size>::Format(Device.clone(), Configuration.clone());
        File_system_type::<Cache_size>::New(Device, Configuration).unwrap()
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
