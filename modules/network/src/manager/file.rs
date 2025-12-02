use crate::{
    GET_HARDWARE_ADDRESS, GET_IP_V4_CONFIGURATION, GET_IP_V6_CONFIGURATION, IS_LINK_UP, Manager,
    StaticIpv4Configuration,
};
use file_system::{
    AttributeOperations, Attributes, BaseOperations, Context, Error, FileOperations, Result, Size,
};

pub(crate) struct FileContext {}

impl BaseOperations for Manager<'_> {
    fn read(&self, _: &mut Context, _: &mut [u8], _: Size) -> Result<usize> {
        Err(Error::UnsupportedOperation)
    }

    fn write(&self, _: &mut Context, _: &[u8], _: Size) -> Result<usize> {
        Err(Error::UnsupportedOperation)
    }

    fn clone_context(&self, _: &Context) -> Result<Context> {
        Err(Error::UnsupportedOperation)
    }

    fn control(
        &self,
        context: &mut Context,
        command: file_system::ControlCommand,
        argument: &mut file_system::ControlArgument,
    ) -> Result<()> {
        let interface_context = context
            .get_private_data_of_type::<crate::manager::context::InterfaceContext>()
            .ok_or(Error::InvalidContext)?;

        match command {
            IS_LINK_UP => {
                let is_up: &mut bool = argument.cast().ok_or(Error::InvalidParameter)?;
                *is_up = interface_context.stack.is_link_up();
            }
            GET_HARDWARE_ADDRESS => {
                let hardware_address: &mut [u8; 6] =
                    argument.cast().ok_or(Error::InvalidParameter)?;
                let address = interface_context.stack.hardware_address();
                hardware_address.copy_from_slice(address.as_bytes());
            }
            GET_IP_V4_CONFIGURATION => {
                let configuration: &mut Option<StaticIpv4Configuration> =
                    argument.cast().ok_or(Error::InvalidParameter)?;

                *configuration = interface_context
                    .stack
                    .config_v4()
                    .map(StaticIpv4Configuration::from_embassy);
            }
            GET_IP_V6_CONFIGURATION => {
                let configuration: &mut Option<crate::StaticIpv6Configuration> =
                    argument.cast().ok_or(Error::InvalidParameter)?;

                *configuration = interface_context
                    .stack
                    .config_v6()
                    .map(crate::StaticIpv6Configuration::from_embassy);
            }
            _ => return Err(Error::UnsupportedOperation),
        }

        Ok(())
    }
}

impl AttributeOperations for Manager<'_> {
    fn get_attributes(&self, context: &mut Context, _: &mut Attributes) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }

    fn set_attributes(&self, _: &mut Context, _: &Attributes) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }
}

impl FileOperations for Manager<'_> {}
