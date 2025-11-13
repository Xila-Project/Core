use crate::{IP, Port, Protocol};
use time::Duration;

use file_system::{LocalFileIdentifierIterator, UniqueFileIdentifier};

use crate::Result;

pub trait SocketDriver: Send + Sync {
    fn get_new_socket_identifier(
        &self,
        iterator: LocalFileIdentifierIterator,
    ) -> Result<Option<UniqueFileIdentifier>>;

    fn close(&self, socket: UniqueFileIdentifier) -> Result<()>;

    fn bind(
        &self,
        ip: IP,
        port: Port,
        protocol: Protocol,
        socket: UniqueFileIdentifier,
    ) -> Result<()>;

    fn connect(&self, ip: IP, port: Port, socket: UniqueFileIdentifier) -> Result<()>;

    fn accept(
        &self,
        socket: UniqueFileIdentifier,
        new_socket: UniqueFileIdentifier,
    ) -> Result<(IP, Port)>;

    fn send(&self, socket: UniqueFileIdentifier, data: &[u8]) -> Result<()>;

    fn send_to(&self, socket: UniqueFileIdentifier, data: &[u8], ip: IP, port: Port) -> Result<()>;

    fn receive(&self, socket: UniqueFileIdentifier, data: &mut [u8]) -> Result<usize>;

    fn receive_from(
        &self,
        socket: UniqueFileIdentifier,
        data: &mut [u8],
    ) -> Result<(usize, IP, Port)>;

    fn get_local_address(&self, socket: UniqueFileIdentifier) -> Result<(IP, Port)>;

    fn get_remote_address(&self, socket: UniqueFileIdentifier) -> Result<(IP, Port)>;

    fn set_send_timeout(&self, socket: UniqueFileIdentifier, timeout: Duration) -> Result<()>;

    fn set_receive_timeout(&self, socket: UniqueFileIdentifier, timeout: Duration) -> Result<()>;

    fn get_send_timeout(&self, socket: UniqueFileIdentifier) -> Result<Option<Duration>>;

    fn get_receive_timeout(&self, socket: UniqueFileIdentifier) -> Result<Option<Duration>>;
}

mod tests {}
