use Executable::Read_data_type;
use File_system::Device_trait;

use crate::Main;

pub struct WASM_device_type;

impl Device_trait for WASM_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<File_system::Size_type> {
        let Read_data: &mut Read_data_type = Buffer
            .try_into()
            .map_err(|_| File_system::Error_type::Invalid_parameter)?;

        *Read_data = Read_data_type::New(Main, 32 * 1024);

        Ok(Read_data.Get_size().into())
    }

    fn Write(&self, _: &[u8]) -> File_system::Result_type<File_system::Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Get_size(&self) -> File_system::Result_type<File_system::Size_type> {
        Ok(size_of::<Read_data_type>().into())
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
