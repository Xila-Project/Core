use std::sync::{Arc, Mutex};

use File_system::{Device_trait, Size_type};
use Graphics::Input_data_type;

use super::Inner_type;

pub struct Pointer_device_type(Arc<Mutex<Inner_type>>);

impl Pointer_device_type {
    pub fn New(Inner: Arc<Mutex<Inner_type>>) -> Self {
        Self(Inner)
    }
}

impl Device_trait for Pointer_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<Size_type> {
        // - Cast the pointer data to the buffer.
        let Data: &mut Input_data_type = Buffer
            .try_into()
            .map_err(|_| File_system::Error_type::Invalid_parameter)?;

        // Copy the pointer data.
        *Data = *self.0.lock().unwrap().Get_pointer_data().unwrap();

        Ok(size_of::<Input_data_type>().into())
    }

    fn Write(&self, _: &[u8]) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Get_size(&self) -> File_system::Result_type<Size_type> {
        Ok(size_of::<Input_data_type>().into())
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }
}
