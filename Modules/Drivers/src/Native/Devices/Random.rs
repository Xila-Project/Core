use File_system::{Device_trait, Size_type};

pub struct Random_device_type;

impl Random_device_type {
    pub fn New() -> Self {
        Self
    }
}

impl Device_trait for Random_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<File_system::Size_type> {
        rand::fill(Buffer);

        Ok(Buffer.len().into())
    }

    fn Write(&self, _Buffer: &[u8]) -> File_system::Result_type<File_system::Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Get_size(&self) -> File_system::Result_type<File_system::Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(
        &self,
        _Position: &File_system::Position_type,
    ) -> File_system::Result_type<File_system::Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }
}
