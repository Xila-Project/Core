use Task::Task_identifier_type;

use crate::{Identifier, Result_type, Socket_identifier_type};

pub struct Manager_type {}

impl Manager_type {
    pub fn Open_socket(Task: Task_identifier_type) -> Result_type<Socket_identifier_type> {
        unimplemented!()
    }

    pub fn Close_socket(Socket: Socket_identifier_type) -> Result_type<()> {
        unimplemented!()
    }

    pub fn Send(Socket: Socket_identifier_type, Data: &[u8]) -> Result_type<()> {
        unimplemented!()
    }

    pub fn Receive(Socket: Socket_identifier_type, Data: &mut [u8]) -> Result_type<usize> {
        unimplemented!()
    }

    pub fn Bind(Socket: Socket_identifier_type, Service: crate::Service_type) -> Result_type<()> {
        unimplemented!()
    }

    pub fn Connect(
        Socket: Socket_identifier_type,
        Service: crate::Service_type,
    ) -> Result_type<()> {
        unimplemented!()
    }

    pub fn Listen(Socket: Socket_identifier_type, Queue_size: usize) -> Result_type<()> {
        unimplemented!()
    }

    pub fn Accept(Socket: Socket_identifier_type) -> Result_type<Socket_identifier_type> {
        unimplemented!()
    }
}
