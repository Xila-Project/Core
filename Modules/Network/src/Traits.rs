use crate::{IP_type, Port_type, Protocol_type};
use Time::Duration_type;

use File_system::{Local_file_identifier_iterator_type, Local_file_identifier_type};

extern crate alloc;

use crate::Result_type;

pub trait Network_socket_driver_trait: Send + Sync {
    fn Get_new_socket_identifier(
        &self,
        Iterator: Local_file_identifier_iterator_type,
    ) -> Result_type<Option<Local_file_identifier_type>>;

    fn Close(&self, Socket: Local_file_identifier_type) -> Result_type<()>;

    fn Bind(
        &self,
        IP: IP_type,
        Port: Port_type,
        Protocol: Protocol_type,
        Socket: Local_file_identifier_type,
    ) -> Result_type<()>;

    fn Connect(
        &self,
        IP: IP_type,
        Port: Port_type,
        Socket: Local_file_identifier_type,
    ) -> Result_type<()>;

    fn Accept(
        &self,
        Socket: Local_file_identifier_type,
        New_socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)>;

    fn Send(&self, Socket: Local_file_identifier_type, Data: &[u8]) -> Result_type<()>;

    fn Send_to(
        &self,
        Socket: Local_file_identifier_type,
        Data: &[u8],
        IP: IP_type,
        Port: Port_type,
    ) -> Result_type<()>;

    fn Receive(&self, Socket: Local_file_identifier_type, Data: &mut [u8]) -> Result_type<usize>;

    fn Receive_from(
        &self,
        Socket: Local_file_identifier_type,
        Data: &mut [u8],
    ) -> Result_type<(usize, IP_type, Port_type)>;

    fn Get_local_address(
        &self,
        Socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)>;

    fn Get_remote_address(
        &self,
        Socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)>;

    fn Set_send_timeout(
        &self,
        Socket: Local_file_identifier_type,
        Timeout: Duration_type,
    ) -> Result_type<()>;

    fn Set_receive_timeout(
        &self,
        Socket: Local_file_identifier_type,
        Timeout: Duration_type,
    ) -> Result_type<()>;

    fn Get_send_timeout(
        &self,
        Socket: Local_file_identifier_type,
    ) -> Result_type<Option<Duration_type>>;

    fn Get_receive_timeout(
        &self,
        Socket: Local_file_identifier_type,
    ) -> Result_type<Option<Duration_type>>;
}

mod Tests {}
