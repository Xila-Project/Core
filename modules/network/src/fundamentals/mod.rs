mod cidr;
mod duration;
mod ip;
mod port;
mod udp;

pub use cidr::*;
pub use duration::*;
pub use ip::*;
pub use port::*;
pub use udp::*;

pub type DNSQueryKind = embassy_net::dns::DnsQueryType;

pub const fn into_embassy_remote_ip_endpoint(ip: IP, port: Port) -> embassy_net::IpEndpoint {
    embassy_net::IpEndpoint::new(ip.into_embassy_address(), port.into_inner())
}

pub const fn into_embassy_local_ip_endpoint(
    address: Option<IP>,
    port: Port,
) -> embassy_net::IpListenEndpoint {
    let address = match address {
        Some(ip) => Some(ip.into_embassy_address()),
        None => None,
    };

    embassy_net::IpListenEndpoint {
        addr: address,
        port: port.into_inner(),
    }
}
