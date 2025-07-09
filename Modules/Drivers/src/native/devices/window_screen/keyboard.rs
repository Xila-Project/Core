use std::sync::{Arc, Mutex};

use file_system::{Device_trait, Size_type};
use graphics::Input_data_type;

use super::Inner_type;

pub struct Keyboard_device_type(Arc<Mutex<Inner_type>>);

impl Keyboard_device_type {
    pub fn new(inner: Arc<Mutex<Inner_type>>) -> Self {
        Self(inner)
    }
}

impl Device_trait for Keyboard_device_type {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result_type<Size_type> {
        // - Cast
        let data: &mut Input_data_type = buffer
            .try_into()
            .map_err(|_| file_system::Error_type::Invalid_parameter)?;

        if let Some((state, key, r#continue)) = self.0.lock().unwrap().pop_keyboard_data() {
            data.set_state(state);
            data.set_key(key);
            data.set_continue(r#continue);
        }

        Ok(size_of::<Input_data_type>().into())
    }

    fn write(&self, _: &[u8]) -> file_system::Result_type<Size_type> {
        Err(file_system::Error_type::Unsupported_operation)
    }

    fn get_size(&self) -> file_system::Result_type<Size_type> {
        Ok(size_of::<Input_data_type>().into())
    }

    fn set_position(&self, _: &file_system::Position_type) -> file_system::Result_type<Size_type> {
        Err(file_system::Error_type::Unsupported_operation)
    }

    fn flush(&self) -> file_system::Result_type<()> {
        Ok(())
    }
}
