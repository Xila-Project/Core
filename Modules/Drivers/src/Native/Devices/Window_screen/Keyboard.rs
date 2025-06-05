use std::sync::{Arc, Mutex};

use File_system::{Device_trait, Size_type};
use Graphics::Input_data_type;

use super::Inner_type;

pub struct Keyboard_device_type(Arc<Mutex<Inner_type>>);

impl Keyboard_device_type {
    pub fn New(Inner: Arc<Mutex<Inner_type>>) -> Self {
        Self(Inner)
    }
}

impl Device_trait for Keyboard_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<Size_type> {
        // - Cast
        let Data: &mut Input_data_type = Buffer
            .try_into()
            .map_err(|_| File_system::Error_type::Invalid_parameter)?;

        if let Some((State, Key, Continue)) = self.0.lock().unwrap().Pop_keyboard_data() {
            Data.Set_state(State);
            Data.Set_key(Key);
            Data.Set_continue(Continue);
        }

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
