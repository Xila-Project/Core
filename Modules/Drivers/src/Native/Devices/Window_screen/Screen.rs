use std::sync::{Arc, Mutex};

use File_system::{Device_trait, Size_type};
use Graphics::{Screen_read_data_type, Screen_write_data_type};

use super::Inner_type;

pub struct Screen_device_type(Arc<Mutex<Inner_type>>);

unsafe impl Sync for Screen_device_type {}

unsafe impl Send for Screen_device_type {}

impl Screen_device_type {
    pub fn New(Inner: Arc<Mutex<Inner_type>>) -> Self {
        Self(Inner)
    }
}

impl Device_trait for Screen_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<File_system::Size_type> {
        let Data: &mut Screen_read_data_type = Buffer
            .try_into()
            .map_err(|_| File_system::Error_type::Invalid_parameter)?;

        let Resolution = self.0.lock()?.Get_resolution().unwrap();

        Data.Set_resolution(Resolution);

        Ok(Size_type::New(size_of::<Self>() as u64))
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<File_system::Size_type> {
        let Data: &Screen_write_data_type = Buffer
            .try_into()
            .map_err(|_| File_system::Error_type::Invalid_parameter)?;

        self.0.lock()?.Draw(Data).unwrap();

        Ok(Size_type::New(size_of::<Self>() as u64))
    }

    fn Get_size(&self) -> File_system::Result_type<File_system::Size_type> {
        Ok(Size_type::New(size_of::<Self>() as u64))
    }

    fn Set_position(
        &self,
        _: &File_system::Position_type,
    ) -> File_system::Result_type<File_system::Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }
}
