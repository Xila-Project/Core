use std::sync::{Arc, Mutex};

use file_system::{Device_trait, Size_type};
use graphics::Input_data_type;

use super::Inner_type;

pub struct Pointer_device_type(Arc<Mutex<Inner_type>>);

impl Pointer_device_type {
    pub fn new(inner: Arc<Mutex<Inner_type>>) -> Self {
        Self(inner)
    }
}

impl Device_trait for Pointer_device_type {
    fn Read(&self, buffer: &mut [u8]) -> File_system::Result_type<Size_type> {
        // - Cast the pointer data to the buffer.
        let data: &mut Input_data_type = buffer
            .try_into()
            .map_err(|_| File_system::Error_type::Invalid_parameter)?;

        // Copy the pointer data.
        *data = *self.0.lock().unwrap().get_pointer_data().unwrap();

        Ok(size_of::<Input_data_type>().into())
    }

    fn Write(&self, _: &[u8]) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn get_size(&self) -> File_system::Result_type<Size_type> {
        Ok(size_of::<Input_data_type>().into())
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }
}
