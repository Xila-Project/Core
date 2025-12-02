use smoltcp::phy::PacketMeta;

use crate::{IP, Port};

#[repr(C)]
pub struct UdpMetadata {
    pub remote_address: IP,
    pub local_address: Option<IP>,

    pub remote_port: Port,

    pub meta: PacketMeta,
}

impl UdpMetadata {
    pub const fn from_embassy_udp_metadata(value: embassy_net::udp::UdpMetadata) -> Self {
        let local_address = match value.local_address {
            Some(addr) => Some(IP::from_embassy_address(addr)),
            None => None,
        };

        Self {
            remote_address: IP::from_embassy_address(value.endpoint.addr),
            local_address,
            remote_port: Port::from_inner(value.endpoint.port),
            meta: value.meta,
        }
    }

    pub const fn to_embassy_udp_metadata(&self) -> embassy_net::udp::UdpMetadata {
        let local_address = match &self.local_address {
            Some(addr) => Some(addr.into_embassy_address()),
            None => None,
        };

        embassy_net::udp::UdpMetadata {
            endpoint: embassy_net::IpEndpoint::new(
                self.remote_address.into_embassy_address(),
                self.remote_port.into_inner(),
            ),
            local_address,
            meta: self.meta,
        }
    }
}
