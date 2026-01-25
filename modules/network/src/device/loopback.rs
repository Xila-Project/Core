use crate::{GET_KIND, InterfaceKind};
use file_system::{
    ControlCommand, ControlCommandIdentifier, DirectBaseOperations, DirectCharacterDevice, Error,
    MountOperations,
};
use shared::AnyByLayout;
pub use smoltcp::phy::Loopback;
use smoltcp::phy::Medium;

pub struct LoopbackControllerDevice;

impl DirectBaseOperations for LoopbackControllerDevice {
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

impl MountOperations for LoopbackControllerDevice {}

impl DirectCharacterDevice for LoopbackControllerDevice {}

pub fn create_loopback_device() -> (Loopback, LoopbackControllerDevice) {
    (Loopback::new(Medium::Ethernet), LoopbackControllerDevice)
}
