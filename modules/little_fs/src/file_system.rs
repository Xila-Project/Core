use core::mem::MaybeUninit;

use alloc::{boxed::Box, ffi::CString, vec::Vec};
use file_system::{
    AttributeOperations, Attributes, BaseOperations, Context, DirectBlockDevice,
    DirectoryOperations, FileOperations, FileSystemOperations, Flags, Path, Size,
};
use futures::block_on;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use crate::{File, attributes::InternalAttributes, configuration::take_device_from_configuration};

use super::{Configuration, Directory, convert_result, littlefs};

use file_system::{Error, Result};

pub struct FileSystem {
    file_system: Mutex<CriticalSectionRawMutex, littlefs::lfs_t>,
    cache_size: usize,
}

impl FileSystem {
    pub fn new(device: &'static dyn DirectBlockDevice, cache_size: usize) -> Result<Self> {
        let block_size = device.get_block_size().map_err(|_| Error::InputOutput)?;
        let size = device.get_size().map_err(|_| Error::InputOutput)?;

        let configuration: littlefs::lfs_config =
            Configuration::new(device, block_size, size as usize, cache_size, cache_size)
                .ok_or(Error::InvalidParameter)?
                .try_into()
                .map_err(|_| Error::InvalidParameter)?;

        let configuration = Box::new(configuration);

        let mut file_system = MaybeUninit::<littlefs::lfs_t>::uninit();

        convert_result(unsafe {
            littlefs::lfs_mount(
                file_system.as_mut_ptr() as *mut _,
                Box::into_raw(configuration),
            )
        })?;

        Ok(Self {
            file_system: Mutex::new(unsafe { file_system.assume_init() }),
            cache_size,
        })
    }

    pub fn format(device: &impl DirectBlockDevice, cache_size: usize) -> Result<()> {
        let block_size = device.get_block_size().map_err(|_| Error::InputOutput)?;
        let size = device.get_size().map_err(|_| Error::InputOutput)?;

        let configuration: littlefs::lfs_config =
            Configuration::new(device, block_size, size as usize, cache_size, cache_size)
                .ok_or(Error::InvalidParameter)?
                .try_into()
                .map_err(|_| Error::InvalidParameter)?;

        let configuration = Box::new(configuration);

        let mut file_system = MaybeUninit::<littlefs::lfs_t>::uninit();

        convert_result(unsafe {
            littlefs::lfs_format(file_system.as_mut_ptr(), Box::into_raw(configuration))
        })?;

        Ok(())
    }

    pub fn operation<T>(
        &self,
        operation: impl FnOnce(&mut littlefs::lfs_t) -> Result<T>,
    ) -> Result<T> {
        let mut file_system = block_on(self.file_system.lock());

        operation(&mut file_system)
    }

    pub fn operation_with_context<I: 'static, T>(
        &self,
        context: &mut Context,
        operation: impl FnOnce(&mut littlefs::lfs_t, &mut I) -> Result<T>,
    ) -> Result<T> {
        let mut file_system = block_on(self.file_system.lock());

        let mut file = context
            .get_private_data_of_type::<I>()
            .ok_or(Error::InvalidParameter)?;

        operation(&mut file_system, &mut file)
    }
}

unsafe impl Send for FileSystem {}
unsafe impl Sync for FileSystem {}

impl FileSystemOperations for FileSystem {
    fn rename(&self, source: &Path, destination: &Path) -> Result<()> {
        self.operation(|file_system| {
            let source = CString::new(source.as_str()).map_err(|_| Error::InvalidParameter)?;

            let destination =
                CString::new(destination.as_str()).map_err(|_| Error::InvalidParameter)?;

            convert_result(unsafe {
                littlefs::lfs_rename(file_system as *mut _, source.as_ptr(), destination.as_ptr())
            })?;

            Ok(())
        })
    }

    fn remove(&self, path: &Path) -> Result<()> {
        let path = CString::new(path.as_str()).map_err(|_| Error::InvalidParameter)?;

        self.operation(|file_system| {
            convert_result(unsafe { littlefs::lfs_remove(file_system as *mut _, path.as_ptr()) })?;

            Ok(())
        })
    }

    fn create_directory(&self, path: &Path) -> Result<()> {
        let path = CString::new(path.as_str()).unwrap();

        self.operation(|file_system| {
            convert_result(unsafe {
                littlefs::lfs_mkdir(&mut *file_system as *mut _, path.as_ptr())
            })?;

            Ok(())
        })
    }

    fn lookup_directory(&self, context: &mut Context, path: &Path) -> Result<()> {
        self.operation(|file_system| {
            let directory = Directory::lookup(file_system, path)?;

            context.set_private_data(Box::new(directory));

            Ok(())
        })
    }

    fn lookup_file(&self, context: &mut Context, path: &Path, flags: Flags) -> Result<()> {
        self.operation(|file_system| {
            let file = File::lookup(file_system, path, flags, self.cache_size)?;
            context.set_private_data(Box::new(file));
            Ok(())
        })
    }

    fn create_file(&self, path: &Path) -> Result<()> {
        self.operation(|file_system| File::create(file_system, path))
    }

    fn get_attributes(&self, path: &Path, attributes: &mut Attributes) -> Result<()> {
        self.operation(|file_system| {
            let path = CString::new(path.as_str()).map_err(|_| Error::InvalidParameter)?;

            let mut internal_attributes =
                unsafe { InternalAttributes::new_uninitialized().assume_init() };

            convert_result(unsafe {
                littlefs::lfs_getattr(
                    &mut *file_system as *mut _,
                    path.as_ptr(),
                    InternalAttributes::IDENTIFIER,
                    &mut internal_attributes as *mut _ as *mut _,
                    size_of::<InternalAttributes>() as u32,
                )
            })?;

            internal_attributes.into_attributes(attributes)?;

            Ok(())
        })
    }

    fn set_attributes(&self, path: &Path, attributes: &Attributes) -> Result<()> {
        self.operation(|file_system| {
            let path = CString::new(path.as_str()).map_err(|_| Error::InvalidParameter)?;

            let mut internal_attributes =
                unsafe { InternalAttributes::new_uninitialized().assume_init() };

            if !attributes.get_mask().are_all_set() {
                convert_result(unsafe {
                    littlefs::lfs_getattr(
                        &mut *file_system as *mut _,
                        path.as_ptr(),
                        InternalAttributes::IDENTIFIER,
                        &mut internal_attributes as *mut _ as *mut _,
                        size_of::<Attributes>() as u32,
                    )
                })?;
            }

            internal_attributes.from_attributes(attributes)?;

            convert_result(unsafe {
                littlefs::lfs_setattr(
                    &mut *file_system as *mut _,
                    path.as_ptr(),
                    InternalAttributes::IDENTIFIER,
                    attributes as *const _ as *const _,
                    size_of::<Attributes>() as u32,
                )
            })?;

            Ok(())
        })
    }
}

impl AttributeOperations for FileSystem {
    fn get_attributes(&self, context: &mut Context, attributes: &mut Attributes) -> Result<()> {
        if let Some(file) = context.get_private_data_of_type::<File>() {
            file.get_attributes(attributes)
        } else if let Some(directory) = context.get_private_data_of_type::<Directory>() {
            directory.get_attributes(attributes)
        } else {
            Err(Error::InvalidParameter)
        }
    }

    fn set_attributes(&self, context: &mut Context, attributes: &Attributes) -> Result<()> {
        if let Some(file) = context.get_private_data_of_type::<File>() {
            file.set_attributes(attributes)
        } else if let Some(directory) = context.get_private_data_of_type::<Directory>() {
            directory.set_attributes(attributes)
        } else {
            Err(Error::InvalidParameter)
        }
    }
}

impl BaseOperations for FileSystem {
    fn read(
        &self,
        context: &mut Context,
        buffer: &mut [u8],
        absolute_position: file_system::Size,
    ) -> Result<usize> {
        self.operation_with_context(context, |file_system, file: &mut File| {
            file.read(file_system, buffer, absolute_position)
        })
    }

    fn write(
        &self,
        context: &mut Context,
        buffer: &[u8],
        absolute_position: file_system::Size,
    ) -> Result<usize> {
        self.operation_with_context(context, |file_system, file: &mut File| {
            file.write(file_system, buffer, absolute_position)
        })
    }

    fn set_position(
        &self,
        context: &mut Context,
        position: &file_system::Position,
    ) -> Result<Size> {
        self.operation_with_context(context, |file_system, file: &mut File| {
            file.set_position(file_system, position)
        })
    }

    fn flush(&self, context: &mut Context) -> Result<()> {
        self.operation_with_context(context, |file_system, file: &mut File| {
            file.flush(file_system)
        })
    }

    fn clone_context(&self, context: &mut Context) -> Result<Context> {
        if let Some(file) = context.get_private_data_of_type::<File>() {
            Ok(Context::new(Some(file.clone())))
        } else if let Some(directory) = context.get_private_data_of_type::<Directory>() {
            Ok(Context::new(Some(directory.clone())))
        } else {
            return Err(Error::InvalidParameter);
        }
    }

    fn close(&self, context: &mut Context) -> Result<()> {
        self.operation(|file_system| {
            let file = context
                .take_private_data_of_type::<File>()
                .ok_or(Error::InvalidParameter)?;

            file.close(file_system)?;

            Ok(())
        })
    }
}

impl FileOperations for FileSystem {}

impl DirectoryOperations for FileSystem {
    fn read(&self, context: &mut file_system::Context) -> Result<Option<file_system::Entry>> {
        self.operation_with_context(context, |file_system, directory: &mut Directory| {
            directory.read(file_system)
        })
    }

    fn set_position(
        &self,
        context: &mut file_system::Context,
        position: file_system::Size,
    ) -> Result<()> {
        self.operation_with_context(context, |file_system, directory: &mut Directory| {
            directory.set_position(file_system, position)
        })
    }

    fn get_position(&self, context: &mut file_system::Context) -> Result<file_system::Size> {
        self.operation_with_context(context, |file_system, directory: &mut Directory| {
            directory.get_position(file_system)
        })
    }

    fn rewind(&self, context: &mut file_system::Context) -> Result<()> {
        self.operation_with_context(context, |file_system, directory: &mut Directory| {
            directory.rewind(file_system)
        })
    }

    fn close(&self, context: &mut file_system::Context) -> Result<()> {
        self.operation(|file_system| {
            let directory = context
                .take_private_data_of_type::<Directory>()
                .ok_or(Error::InvalidParameter)?;

            directory.close(file_system)?;

            Ok(())
        })
    }
}

impl Drop for FileSystem {
    fn drop(&mut self) {
        let _ = self.operation(|file_system| {
            let configuration =
                unsafe { Box::from_raw(file_system.cfg as *mut littlefs::lfs_config) };

            let _read_buffer = unsafe {
                Vec::from_raw_parts(
                    configuration.read_buffer as *mut u8,
                    0,
                    configuration.cache_size as usize,
                )
            };
            let _write_buffer = unsafe {
                Vec::from_raw_parts(
                    configuration.prog_buffer as *mut u8,
                    0,
                    configuration.cache_size as usize,
                )
            };
            let _look_ahead_buffer = unsafe {
                Vec::from_raw_parts(
                    configuration.lookahead_buffer as *mut u8,
                    0,
                    configuration.lookahead_size as usize,
                )
            };

            let _device = unsafe { take_device_from_configuration(&*configuration) };

            unsafe {
                littlefs::lfs_unmount(&mut *file_system as *mut _);
            }

            Ok(())
        });
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use alloc::sync::Arc;
    use file_system::{MemoryDevice, create_device};
    use task::test;

    use super::*;

    const CACHE_SIZE: usize = 256;

    fn initialize() -> FileSystem {
        let _ = users::initialize();

        task::initialize();

        let _ = time::initialize(create_device!(drivers_native::TimeDriver::new()));

        let mock_device = MemoryDevice::<512>::new(2048 * 512);

        let device = Device::new(Arc::new(mock_device));

        FileSystem::format(device.clone(), CACHE_SIZE).unwrap();

        FileSystem::new(device, CACHE_SIZE).unwrap()
    }

    #[test]
    async fn test_open_close_delete() {
        file_system::tests::test_open_close_delete(initialize()).await;
    }

    #[test]
    async fn test_read_write() {
        file_system::tests::test_read_write(initialize()).await;
    }

    #[test]
    async fn test_move() {
        file_system::tests::test_move(initialize()).await;
    }

    #[test]
    async fn test_set_position() {
        file_system::tests::test_set_position(initialize()).await;
    }

    #[test]
    async fn test_flush() {
        file_system::tests::test_flush(initialize()).await;
    }

    #[test]
    async fn test_set_get_metadata() {
        file_system::tests::test_set_get_metadata(initialize()).await;
    }

    #[test]
    async fn test_read_directory() {
        file_system::tests::test_read_directory(initialize()).await;
    }

    #[test]
    async fn test_set_position_directory() {
        file_system::tests::test_set_position_directory(initialize()).await;
    }

    #[test]
    async fn test_rewind_directory() {
        file_system::tests::test_rewind_directory(initialize()).await;
    }

    #[test]
    async fn test_create_remove_directory() {
        file_system::tests::test_create_remove_directory(initialize()).await;
    }

    #[cfg(feature = "std")]
    #[test]
    async fn test_loader() {
        file_system::tests::test_loader(initialize());
    }
}
