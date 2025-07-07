use crate::{IP_type, Port_type, Protocol_type};
use Time::Duration_type;

use File_system::{Local_file_identifier_iterator_type, Local_file_identifier_type};

use crate::Result_type;

pub trait Network_socket_driver_trait: Send + Sync {
    fn get_new_socket_identifier(
        &self,
        iterator: Local_file_identifier_iterator_type,
    ) -> Result_type<Option<Local_file_identifier_type>>;

    fn Close(&self, Socket: Local_file_identifier_type) -> Result_type<()>;

    fn Bind(
        &self,
        ip: IP_type,
        port: Port_type,
        protocol: Protocol_type,
        socket: Local_file_identifier_type,
    ) -> Result_type<()>;

    fn Connect(
        &self,
        ip: IP_type,
        port: Port_type,
        socket: Local_file_identifier_type,
    ) -> Result_type<()>;

    fn Accept(
        &self,
        socket: Local_file_identifier_type,
        new_socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)>;

    fn Send(&self, Socket: Local_file_identifier_type, Data: &[u8]) -> Result_type<()>;

    fn Send_to(
        &self,
        socket: Local_file_identifier_type,
        data: &[u8],
        ip: IP_type,
        port: Port_type,
    ) -> Result_type<()>;

    fn Receive(&self, Socket: Local_file_identifier_type, Data: &mut [u8]) -> Result_type<usize>;

    fn Receive_from(
        &self,
        socket: Local_file_identifier_type,
        data: &mut [u8],
    ) -> Result_type<(usize, IP_type, Port_type)>;

    fn Get_local_address(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)>;

    fn Get_remote_address(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)>;

    fn Set_send_timeout(
        &self,
        socket: Local_file_identifier_type,
        timeout: Duration_type,
    ) -> Result_type<()>;

    fn Set_receive_timeout(
        &self,
        socket: Local_file_identifier_type,
        timeout: Duration_type,
    ) -> Result_type<()>;

    fn Get_send_timeout(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<Option<Duration_type>>;

    fn Get_receive_timeout(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<Option<Duration_type>>;
}

mod Tests {}
