use crate::{Port, Result, UdpMetadata, UdpSocketContext};

pub struct UdpSocket<'a> {
    context: UdpSocketContext<'a>,
}

impl<'a> UdpSocket<'a> {
    pub fn new(context: UdpSocketContext<'a>) -> Self {
        Self { context }
    }

    pub fn bind(&mut self, port: Port) -> Result<()> {
        self.context.socket.bind(port.into_inner())?;

        Ok(())
    }

    pub async fn read_from(&mut self, buffer: &mut [u8]) -> Result<(usize, UdpMetadata)> {
        let (size, metadata) = self.context.socket.recv_from(buffer).await?;

        let metadata = UdpMetadata::from_embassy_udp_metadata(metadata);

        Ok((size, metadata))
    }

    pub async fn write_to(&mut self, buffer: &[u8], metadata: &UdpMetadata) -> Result<()> {
        let embassy_metadata = metadata.to_embassy_udp_metadata();

        self.context
            .socket
            .send_to(buffer, embassy_metadata)
            .await?;

        Ok(())
    }

    pub async fn close(mut self) -> Result<()> {
        self.context.socket.close();

        Ok(())
    }

    pub fn get_payload_receive_capacity(&self) -> Result<usize> {
        Ok(self.context.socket.payload_recv_capacity())
    }

    pub fn get_payload_transmit_capacity(&self) -> Result<usize> {
        Ok(self.context.socket.payload_send_capacity())
    }

    pub fn set_hop_limit(&mut self, hop_limit: Option<u8>) -> Result<()> {
        self.context.socket.set_hop_limit(hop_limit);

        Ok(())
    }
}
