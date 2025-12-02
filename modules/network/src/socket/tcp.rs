use file_system::{Flags, StateFlags};

use crate::{
    Duration, IP, Manager, Port, Result, TcpSocketContext, into_embassy_local_ip_endpoint,
    into_embassy_remote_ip_endpoint,
};

pub struct TcpSocket<'a> {
    manager: &'a Manager<'a>,
    context: TcpSocketContext<'a>,
}

impl<'a> TcpSocket<'a> {
    pub fn new(manager: &'a Manager<'a>, context: TcpSocketContext<'a>) -> Self {
        TcpSocket { manager, context }
    }

    pub async fn set_timeout(&mut self, timeout: Option<Duration>) -> Result<()> {
        self.context
            .socket
            .set_timeout(timeout.map(|d| d.into_embassy()));
        Ok(())
    }

    pub async fn accept(&mut self, address: Option<IP>, port: Port) -> Result<()> {
        let endpoint = into_embassy_local_ip_endpoint(address, port);

        Ok(self.context.socket.accept(endpoint).await?)
    }

    pub async fn connect(&mut self, ip: IP, port: Port) -> Result<()> {
        let endpoint = into_embassy_remote_ip_endpoint(ip, port);

        Ok(self.context.socket.connect(endpoint).await?)
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        Ok(self.context.socket.read(buffer).await?)
    }

    pub async fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        Ok(self.context.socket.write(buffer).await?)
    }

    pub async fn close(mut self) -> Result<()> {
        self.context.socket.close();
        Ok(())
    }

    pub async fn close_write(&mut self) -> Result<()> {
        self.context.socket.close();
        Ok(())
    }
}
