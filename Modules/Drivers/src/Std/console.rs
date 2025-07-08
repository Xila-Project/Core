use std::io::{stderr, stdin, stdout, Read, Write};

use file_system::{Device_trait, Size_type};

use crate::Std::IO::Map_error;

pub struct Standard_in_device_type;

impl Device_trait for Standard_in_device_type {
    fn Read(&self, buffer: &mut [u8]) -> File_system::Result_type<Size_type> {
        #[allow(clippy::unused_io_amount)]
        stdin().read(buffer).unwrap();

        Ok(Size_type::New(buffer.len() as u64))
    }

    fn Write(&self, _: &[u8]) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn get_size(&self) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }

    fn is_a_terminal(&self) -> bool {
        true
    }
}

pub struct Standard_out_device_type;

impl Device_trait for Standard_out_device_type {
    fn Read(&self, _: &mut [u8]) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(
            stdout().write(Buffer).map_err(Map_error)? as u64
        ))
    }

    fn get_size(&self) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        stdout().flush().map_err(Map_error)
    }

    fn is_a_terminal(&self) -> bool {
        true
    }
}

pub struct Standard_error_device_type;

impl Device_trait for Standard_error_device_type {
    fn Read(&self, _: &mut [u8]) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(
            stderr().write(Buffer).map_err(Map_error)? as u64
        ))
    }

    fn get_size(&self) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        stderr().flush().map_err(Map_error)
    }

    fn is_a_terminal(&self) -> bool {
        true
    }
}
