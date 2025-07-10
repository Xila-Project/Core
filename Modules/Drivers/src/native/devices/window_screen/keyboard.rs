use std::sync::{Arc, Mutex};

use file_system::{DeviceTrait, Size};
use graphics::Input_data_type;

use super::Inner;

pub struct KeyboardDeviceType(Arc<Mutex<Inner>>);

impl KeyboardDeviceType {
    pub fn new(inner: Arc<Mutex<Inner>>) -> Self {
        Self(inner)
    }
}

impl DeviceTrait for KeyboardDeviceType {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<Size> {
        // - Cast
        let data: &mut Input_data_type = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        if let Some((state, key, r#continue)) = self.0.lock().unwrap().pop_keyboard_data() {
            data.set_state(state);
            data.set_key(key);
            data.set_continue(r#continue);
        }

        Ok(size_of::<Input_data_type>().into())
    }

    fn write(&self, _: &[u8]) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn get_size(&self) -> file_system::Result<Size> {
        Ok(size_of::<Input_data_type>().into())
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}
