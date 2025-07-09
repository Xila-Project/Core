use crate::{IP_type, Port_type, Protocol_type};
use time::Duration_type;

use file_system::{Local_file_identifier_iterator_type, Local_file_identifier_type};

use crate::Result_type;

pub trait Network_socket_driver_trait: Send + Sync {
    fn get_new_socket_identifier(
        &self,
        iterator: Local_file_identifier_iterator_type,
    ) -> Result_type<Option<Local_file_identifier_type>>;

    fn close(&self, socket: Local_file_identifier_type) -> Result_type<()>;

    fn bind(
        &self,
        ip: IP_type,
        port: Port_type,
        protocol: Protocol_type,
        socket: Local_file_identifier_type,
    ) -> Result_type<()>;

    fn connect(
        &self,
        ip: IP_type,
        port: Port_type,
        socket: Local_file_identifier_type,
    ) -> Result_type<()>;

    fn accept(
        &self,
        socket: Local_file_identifier_type,
        new_socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)>;

    fn send(&self, socket: Local_file_identifier_type, data: &[u8]) -> Result_type<()>;

    fn send_to(
        &self,

        socket: Local_file_identifier_type,
        data: &[u8],
        ip: IP_type,
        port: Port_type,
    ) -> Result_type<()>;

    fn receive(&self, socket: Local_file_identifier_type, data: &mut [u8]) -> Result_type<usize>;

    fn receive_from(
        &self,
        socket: Local_file_identifier_type,
        data: &mut [u8],
    ) -> Result_type<(usize, IP_type, Port_type)>;

    fn get_local_address(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)>;

    fn get_remote_address(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)>;

    fn set_send_timeout(
        &self,
        socket: Local_file_identifier_type,
        timeout: Duration_type,
    ) -> Result_type<()>;

    fn set_receive_timeout(
        &self,
        socket: Local_file_identifier_type,
        timeout: Duration_type,
    ) -> Result_type<()>;

    fn get_send_timeout(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<Option<Duration_type>>;

    fn get_receive_timeout(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<Option<Duration_type>>;
}

mod tests {}
