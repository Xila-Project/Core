use file_system::Path_owned_type;

use network::{IP_type, IPv4_type, IPv6_type, Port_type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Socket_address_type {
    IPv4(IPv4_type, Port_type),
    IPv6(IPv6_type, Port_type),
    Local(Path_owned_type),
}

impl Socket_address_type {
    pub fn into_ip_and_port(self) -> Option<(IP_type, Port_type)> {
        match self {
            Self::IPv4(ip, port) => Some((ip.into(), port)),
            Self::IPv6(ip, port) => Some((ip.into(), port)),
            _ => None,
        }
    }

    pub const fn from_ip_and_port(ip: IP_type, port: Port_type) -> Self {
        match ip {
            IP_type::IPv4(ip) => Self::IPv4(ip, port),
            IP_type::IPv6(ip) => Self::IPv6(ip, port),
        }
    }
}

impl From<(IPv4_type, Port_type)> for Socket_address_type {
    fn from((ip, port): (IPv4_type, Port_type)) -> Self {
        Self::IPv4(ip, port)
    }
}

impl From<(IPv6_type, Port_type)> for Socket_address_type {
    fn from((ip, port): (IPv6_type, Port_type)) -> Self {
        Self::IPv6(ip, port)
    }
}

impl From<Path_owned_type> for Socket_address_type {
    fn from(path: Path_owned_type) -> Self {
        Self::Local(path)
    }
}
