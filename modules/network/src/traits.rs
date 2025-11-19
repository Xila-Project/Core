use crate::{IP, Port, Protocol};
use file_system::Context;
use time::Duration;

use crate::Result;

pub trait SocketDriver: Send + Sync {
    fn close(&self, context: &mut Context) -> Result<()>;

    fn bind(&self, ip: IP, port: Port, protocol: Protocol, context: &mut Context) -> Result<()>;

    fn connect(&self, ip: IP, port: Port, context: &mut Context) -> Result<()>;

    fn accept(&self, context: &mut Context) -> Result<(IP, Port)>;

    fn send(&self, context: &mut Context, data: &[u8]) -> Result<()>;

    fn send_to(&self, context: &mut Context, data: &[u8], ip: IP, port: Port) -> Result<()>;

    fn receive(&self, context: &mut Context, data: &mut [u8]) -> Result<usize>;

    fn receive_from(&self, context: &mut Context, data: &mut [u8]) -> Result<(usize, IP, Port)>;

    fn get_local_address(&self, context: &mut Context) -> Result<(IP, Port)>;

    fn get_remote_address(&self, context: &mut Context) -> Result<(IP, Port)>;

    fn set_send_timeout(&self, context: &mut Context, timeout: Duration) -> Result<()>;

    fn set_receive_timeout(&self, context: &mut Context, timeout: Duration) -> Result<()>;

    fn get_send_timeout(&self, context: &mut Context) -> Result<Option<Duration>>;

    fn get_receive_timeout(&self, context: &mut Context) -> Result<Option<Duration>>;
}

mod tests {}
