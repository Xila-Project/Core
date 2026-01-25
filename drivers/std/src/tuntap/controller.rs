use file_system::{
    ControlCommand, ControlCommandIdentifier, DirectBaseOperations, DirectCharacterDevice, Error,
    MountOperations,
};
use network::{GET_KIND, InterfaceKind};
use shared::AnyByLayout;

pub struct TunTapControllerDevice;

impl DirectBaseOperations for TunTapControllerDevice {
    fn read(&self, _: &mut [u8], _: file_system::Size) -> file_system::Result<usize> {
        Err(Error::UnsupportedOperation)
    }

    fn write(&self, _: &[u8], _: file_system::Size) -> file_system::Result<usize> {
        Err(Error::UnsupportedOperation)
    }

    fn control(
        &self,
        command: ControlCommandIdentifier,
        _: &AnyByLayout,
        output: &mut AnyByLayout,
    ) -> file_system::Result<()> {
        match command {
            GET_KIND::IDENTIFIER => {
                let kind = GET_KIND::cast_output(output)?;
                *kind = InterfaceKind::Ethernet;
                Ok(())
            }
            _ => Err(Error::UnsupportedOperation),
        }
    }
}

impl MountOperations for TunTapControllerDevice {}

impl DirectCharacterDevice for TunTapControllerDevice {}
