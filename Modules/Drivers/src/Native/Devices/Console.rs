use std::io::{stderr, stdin, stdout, Read, Write};

use File_system::{Create_device, Device_trait, Size_type};
use Task::Task_identifier_type;
use Virtual_file_system::Virtual_file_system_type;

use crate::Std::IO::Map_error;

pub struct Standard_in_device_type;

impl Device_trait for Standard_in_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<Size_type> {
        #[allow(clippy::unused_io_amount)]
        stdin().read(Buffer).unwrap();

        Ok(Size_type::New(Buffer.len() as u64))
    }

    fn Write(&self, _: &[u8]) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Get_size(&self) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }

    fn Is_a_terminal(&self) -> bool {
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

    fn Get_size(&self) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(stdout().flush().map_err(Map_error)?)
    }

    fn Is_a_terminal(&self) -> bool {
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

    fn Get_size(&self) -> File_system::Result_type<Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<Size_type> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(stderr().flush().map_err(Map_error)?)
    }

    fn Is_a_terminal(&self) -> bool {
        true
    }
}

pub async fn Mount_devices(
    Task: Task_identifier_type,
    Virtual_file_system: &Virtual_file_system_type<'_>,
) -> Result<(), String> {
    Virtual_file_system
        .Mount_static_device(
            Task,
            &"/Devices/Standard_in",
            Create_device!(Standard_in_device_type),
        )
        .await
        .map_err(|Error| format!("Error adding standard in device: {:?}", Error))?;

    Virtual_file_system
        .Mount_static_device(
            Task,
            &"/Devices/Standard_out",
            Create_device!(Standard_out_device_type),
        )
        .await
        .map_err(|Error| format!("Error adding standard out device: {:?}", Error))?;

    Virtual_file_system
        .Mount_static_device(
            Task,
            &"/Devices/Standard_error",
            Create_device!(Standard_error_device_type),
        )
        .await
        .map_err(|Error| format!("Error adding standard error device: {:?}", Error))?;

    Ok(())
}
