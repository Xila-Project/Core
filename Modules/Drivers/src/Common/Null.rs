use File_system::{Device_trait, Size_type};

pub struct Null_device_type;

impl Device_trait for Null_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<File_system::Size_type> {
        Ok(Size_type::New(Buffer.len() as u64))
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<File_system::Size_type> {
        Ok(Size_type::New(Buffer.len() as u64))
    }

    fn Get_size(&self) -> File_system::Result_type<File_system::Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(
        &self,
        _: &File_system::Position_type,
    ) -> File_system::Result_type<File_system::Size_type> {
        Ok(Size_type::New(0))
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }
}
