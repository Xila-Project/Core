use alloc::string::String;
use file_system::Device_trait;
use futures::block_on;

use crate::terminal::Terminal_type;

impl Device_trait for Terminal_type {
    fn Read(&self, buffer: &mut [u8]) -> file_system::Result_type<file_system::Size_type> {
        block_on(self.read_input(buffer)).map_err(|_| file_system::Error_type::Internal_error)
    }

    fn Write(&self, buffer: &[u8]) -> file_system::Result_type<file_system::Size_type> {
        let string = String::from_utf8_lossy(buffer);

        block_on(self.print(&string)).map_err(|_| file_system::Error_type::Internal_error)?;

        Ok(buffer.len().into())
    }

    fn get_size(&self) -> file_system::Result_type<file_system::Size_type> {
        Ok(0_usize.into())
    }

    fn Set_position(
        &self,
        _: &file_system::Position_type,
    ) -> file_system::Result_type<file_system::Size_type> {
        Err(file_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> file_system::Result_type<()> {
        Ok(())
    }
}
