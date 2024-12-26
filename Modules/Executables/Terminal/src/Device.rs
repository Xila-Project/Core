use File_system::Device_trait;

use crate::Terminal::Terminal_type;

impl Device_trait for Terminal_type {
    fn Read(&self, _: &mut [u8]) -> File_system::Result_type<File_system::Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Read_line(&self, Buffer: &mut String) -> File_system::Result_type<File_system::Size_type> {
        let Size = self
            .Read_input(Buffer)
            .map_err(|_| File_system::Error_type::Internal_error)?;

        Ok(Size.into())
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<File_system::Size_type> {
        let String = String::from_utf8_lossy(Buffer);

        self.Print(&String)
            .map_err(|_| File_system::Error_type::Internal_error)?;

        Ok(Buffer.len().into())
    }

    fn Get_size(&self) -> File_system::Result_type<File_system::Size_type> {
        Ok(0_usize.into())
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
