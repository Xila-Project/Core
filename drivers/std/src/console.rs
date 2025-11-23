use std::io::{Read, Write, stderr, stdin, stdout};

use file_system::{
    ControlArgument, ControlCommand, DirectBaseOperations, DirectCharacterDevice, MountOperations,
    Size, character_device,
};

use crate::io::map_error;

pub struct StandardInDevice;

impl DirectBaseOperations for StandardInDevice {
    fn read(&self, buffer: &mut [u8], _: Size) -> file_system::Result<usize> {
        #[allow(clippy::unused_io_amount)]
        stdin().read(buffer).unwrap();

        Ok(buffer.len() as _)
    }

    fn write(&self, _: &[u8], _: Size) -> file_system::Result<usize> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }

    fn control(
        &self,
        command: ControlCommand,
        argument: &mut ControlArgument,
    ) -> file_system::Result<()> {
        control(command, argument)
    }
}

impl MountOperations for StandardInDevice {}

impl DirectCharacterDevice for StandardInDevice {}

pub struct StandardOutDevice;

impl DirectBaseOperations for StandardOutDevice {
    fn read(&self, _: &mut [u8], _: Size) -> file_system::Result<usize> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn write(&self, buffer: &[u8], _: Size) -> file_system::Result<usize> {
        Ok(stdout().write(buffer).map_err(map_error)? as _)
    }

    fn flush(&self) -> file_system::Result<()> {
        stdout().flush().map_err(map_error)
    }

    fn control(
        &self,
        command: ControlCommand,
        argument: &mut ControlArgument,
    ) -> file_system::Result<()> {
        control(command, argument)
    }
}

impl MountOperations for StandardOutDevice {}

impl DirectCharacterDevice for StandardOutDevice {}

pub struct StandardErrorDevice;

impl DirectBaseOperations for StandardErrorDevice {
    fn read(&self, _: &mut [u8], _: Size) -> file_system::Result<usize> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn write(&self, buffer: &[u8], _: Size) -> file_system::Result<usize> {
        Ok(stderr().write(buffer).map_err(map_error)? as _)
    }

    fn flush(&self) -> file_system::Result<()> {
        stderr().flush().map_err(map_error)
    }

    fn control(
        &self,
        command: ControlCommand,
        argument: &mut ControlArgument,
    ) -> file_system::Result<()> {
        control(command, argument)
    }
}

impl MountOperations for StandardErrorDevice {}

impl DirectCharacterDevice for StandardErrorDevice {}

fn control(command: ControlCommand, argument: &mut ControlArgument) -> file_system::Result<()> {
    match command {
        character_device::IS_A_TERMINAL => {
            *argument
                .cast::<bool>()
                .ok_or(file_system::Error::InvalidParameter)? = true;

            Ok(())
        }
        _ => Err(file_system::Error::UnsupportedOperation),
    }
}
