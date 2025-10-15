use alloc::string::String;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use file_system::{DeviceTrait, Path, Size};
use futures::block_on;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::*;

fn map_error_to_file_system_error(value: JsValue) -> file_system::Error {
    log::Error!("Drive error: {:?}", value);

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
    options: FileSystemReadWriteOptions,
    estimate: StorageEstimate,
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

        let estimate = JsFuture::from(storage.estimate().map_err(map_error_to_string)?)
            .await
            .map_err(map_error_to_string)?
            .dyn_into::<StorageEstimate>()
            .map_err(map_error_to_string)?;

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

        Ok(Self {
            handle,
            options: FileSystemReadWriteOptions::new(),
            estimate,
        })
    }

    async fn write_async(&mut self, data: &[u8]) -> file_system::Result<Size> {
        let size = self
            .handle
            .write_with_u8_array_and_options(data, &self.options)
            .map_err(map_error_to_file_system_error)?;
        let size = Size::new(size as u64);
        self.increment_position(size.as_u64());
        Ok(size)
    }

    async fn read_async(&mut self, data: &mut [u8]) -> file_system::Result<Size> {
        let size = self
            .handle
            .read_with_u8_array_and_options(data, &self.options)
            .map_err(|_| file_system::Error::InputOutput)?;
        let size = Size::new(size as u64);
        self.increment_position(size.as_u64());
        Ok(size)
    }

    fn increment_position(&mut self, offset: u64) {
        self.options
            .set_at((self.get_absolute_position() + offset) as _);
    }

    pub fn get_absolute_position(&self) -> u64 {
        self.options.get_at().unwrap_or_default() as u64
    }

    pub fn set_absolute_position(&mut self, pos: u64) {
        self.options.set_at(pos as _);
    }

    pub fn get_estimate(&self) -> &StorageEstimate {
        &self.estimate
    }
}

pub struct DriveDevice(RwLock<CriticalSectionRawMutex, Inner>);

impl DriveDevice {
    pub fn new(name: &Path) -> Self {
        let inner = block_on(Inner::new(name)).expect("Failed to create drive device");
        Self(RwLock::new(inner))
    }
}

impl DeviceTrait for DriveDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<file_system::Size> {
        let mut inner = block_on(self.0.write());
        block_on(inner.read_async(buffer))
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result<file_system::Size> {
        let mut inner = block_on(self.0.write());
        block_on(inner.write_async(buffer))
    }

    fn get_size(&self) -> file_system::Result<file_system::Size> {
        let inner = block_on(self.0.read());
        let estimate = inner.get_estimate();
        Ok(file_system::Size::new(
            estimate.get_quota().unwrap_or(0.0) as u64
        ))
    }

    fn set_position(
        &self,
        position: &file_system::Position,
    ) -> file_system::Result<file_system::Size> {
        let mut inner = block_on(self.0.write());
        match position {
            file_system::Position::Start(pos) => {
                inner.set_absolute_position(*pos);
            }
            file_system::Position::End(pos) => {
                let current_pos = inner.get_absolute_position();
                inner.set_absolute_position(current_pos.saturating_sub(*pos as u64));
            }
            file_system::Position::Current(pos) => {
                let current_pos = inner.get_absolute_position();
                inner.set_absolute_position(current_pos.saturating_add(*pos as u64));
            }
        }
        Ok(file_system::Size::new(inner.get_absolute_position()))
    }

    fn flush(&self) -> file_system::Result<()> {
        let inner = block_on(self.0.write());
        inner.handle.flush().map_err(map_error_to_file_system_error)
    }
}
