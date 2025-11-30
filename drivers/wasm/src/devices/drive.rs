use alloc::string::{String, ToString};
use file_system::{DirectBaseOperations, Path, Size};
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use task::block_on;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::*;

fn map_error_to_file_system_error(value: JsValue) -> file_system::Error {
    log::error!("Drive error: {:?}", value);

    file_system::Error::InputOutput
}

fn map_error_to_string(value: JsValue) -> String {
    if let Some(string) = value.as_string() {
        return string;
    }

    "Unknown error".to_string()
}

struct Inner {
    handle: FileSystemSyncAccessHandle,
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.handle.close();
    }
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

impl Inner {
    pub async fn new(name: &Path) -> Result<Self, String> {
        let storage = web_sys::window()
            .ok_or("Failed to get window")?
            .navigator()
            .storage();

        let directory = JsFuture::from(storage.get_directory())
            .await
            .map_err(map_error_to_string)?
            .dyn_into::<FileSystemDirectoryHandle>()
            .map_err(map_error_to_string)?;

        let options = FileSystemGetFileOptions::new();
        options.set_create(true);

        let asynchronous_handle =
            JsFuture::from(directory.get_file_handle_with_options(name.as_str(), &options))
                .await
                .map_err(map_error_to_string)?
                .dyn_into::<FileSystemFileHandle>()
                .map_err(map_error_to_string)?;

        let handle = JsFuture::from(asynchronous_handle.create_sync_access_handle())
            .await
            .map_err(map_error_to_string)?
            .dyn_into::<FileSystemSyncAccessHandle>()
            .map_err(map_error_to_string)?;

        Ok(Self { handle })
    }

    async fn write_async(&mut self, data: &[u8], position: Size) -> file_system::Result<usize> {
        let options = FileSystemReadWriteOptions::new();
        options.set_at(position as _);

        let size = self
            .handle
            .write_with_u8_array_and_options(data, &options)
            .map_err(map_error_to_file_system_error)?;

        Ok(size as _)
    }

    async fn read_async(&mut self, data: &mut [u8], position: Size) -> file_system::Result<usize> {
        let options = FileSystemReadWriteOptions::new();
        options.set_at(position as _);

        let size = self
            .handle
            .read_with_u8_array_and_options(data, &options)
            .map_err(|_| file_system::Error::InputOutput)?;

        Ok(size as _)
    }
}

pub struct DriveDevice(RwLock<CriticalSectionRawMutex, Inner>);

impl DriveDevice {
    pub fn new(name: &Path) -> Self {
        let inner = block_on(Inner::new(name)).expect("Failed to create drive device");
        Self(RwLock::new(inner))
    }
}

impl DirectBaseOperations for DriveDevice {
    fn read(&self, buffer: &mut [u8], position: Size) -> file_system::Result<usize> {
        let mut inner = block_on(self.0.write());
        block_on(inner.read_async(buffer, position))
    }

    fn write(&self, buffer: &[u8], position: Size) -> file_system::Result<usize> {
        let mut inner = block_on(self.0.write());
        block_on(inner.write_async(buffer, position))
    }

    fn set_position(
        &self,
        current_position: Size,
        _: &file_system::Position,
    ) -> file_system::Result<Size> {
        Ok(current_position) // TODO: Implement seek if needed
    }

    fn flush(&self) -> file_system::Result<()> {
        let inner = block_on(self.0.write());
        inner.handle.flush().map_err(map_error_to_file_system_error)
    }
}
