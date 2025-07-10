use crate::{Port, Protocol, IP};
use time::Duration;

use file_system::{LocalFileIdentifier, LocalFileIdentifierIterator};

use crate::Result;

pub trait SocketDriver: Send + Sync {
    fn get_new_socket_identifier(
        &self,
        iterator: LocalFileIdentifierIterator,
    ) -> Result<Option<LocalFileIdentifier>>;

    fn close(&self, socket: LocalFileIdentifier) -> Result<()>;

    fn bind(
        &self,
        ip: IP,
        port: Port,
        protocol: Protocol,
        socket: LocalFileIdentifier,
    ) -> Result<()>;

    fn connect(&self, ip: IP, port: Port, socket: LocalFileIdentifier) -> Result<()>;

    fn accept(
        &self,
        socket: LocalFileIdentifier,
        new_socket: LocalFileIdentifier,
    ) -> Result<(IP, Port)>;

    fn send(&self, socket: LocalFileIdentifier, data: &[u8]) -> Result<()>;

    fn send_to(&self, socket: LocalFileIdentifier, data: &[u8], ip: IP, port: Port) -> Result<()>;

    fn receive(&self, socket: LocalFileIdentifier, data: &mut [u8]) -> Result<usize>;

    fn receive_from(
        &self,
        socket: LocalFileIdentifier,
        data: &mut [u8],
    ) -> Result<(usize, IP, Port)>;

    fn get_local_address(&self, socket: LocalFileIdentifier) -> Result<(IP, Port)>;

    fn get_remote_address(&self, socket: LocalFileIdentifier) -> Result<(IP, Port)>;

    fn set_send_timeout(&self, socket: LocalFileIdentifier, timeout: Duration) -> Result<()>;

    fn set_receive_timeout(&self, socket: LocalFileIdentifier, timeout: Duration) -> Result<()>;

    fn get_send_timeout(&self, socket: LocalFileIdentifier) -> Result<Option<Duration>>;

    fn get_receive_timeout(&self, socket: LocalFileIdentifier) -> Result<Option<Duration>>;
}

mod tests {}
