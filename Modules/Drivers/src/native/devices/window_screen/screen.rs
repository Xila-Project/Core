use std::sync::{Arc, Mutex};

use file_system::{Device_trait, Size_type};
use graphics::{Screen_read_data_type, Screen_write_data_type};

use super::Inner_type;

pub struct Screen_device_type(Arc<Mutex<Inner_type>>);

unsafe impl Sync for Screen_device_type {}

unsafe impl Send for Screen_device_type {}

impl Screen_device_type {
    pub fn new(inner: Arc<Mutex<Inner_type>>) -> Self {
        Self(inner)
    }
}

impl Device_trait for Screen_device_type {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result_type<file_system::Size_type> {
        let data: &mut Screen_read_data_type = buffer
            .try_into()
            .map_err(|_| file_system::Error_type::Invalid_parameter)?;

        let resolution = self.0.lock().unwrap().get_resolution().unwrap();

        data.set_resolution(resolution);

        Ok(Size_type::new(size_of::<Self>() as u64))
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result_type<file_system::Size_type> {
        let data: &Screen_write_data_type = buffer
            .try_into()
            .map_err(|_| file_system::Error_type::Invalid_parameter)?;

        self.0.lock().unwrap().draw(data).unwrap();

        Ok(Size_type::new(size_of::<Self>() as u64))
    }

    fn get_size(&self) -> file_system::Result_type<file_system::Size_type> {
        Ok(Size_type::new(size_of::<Self>() as u64))
    }

    fn set_position(
        &self,
        _: &file_system::Position_type,
    ) -> file_system::Result_type<file_system::Size_type> {
        Err(file_system::Error_type::Unsupported_operation)
    }

    fn flush(&self) -> file_system::Result_type<()> {
        Ok(())
    }
}
