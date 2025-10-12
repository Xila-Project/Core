use file_system::PathOwned;

use network::{IP, IPv4, IPv6, Port};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SockerAddress {
    IPv4(IPv4, Port),
    IPv6(IPv6, Port),
    Local(PathOwned),
}

impl SockerAddress {
    pub fn into_ip_and_port(self) -> Option<(IP, Port)> {
        match self {
            Self::IPv4(ip, port) => Some((ip.into(), port)),
            Self::IPv6(ip, port) => Some((ip.into(), port)),
            _ => None,
        }
    }

    pub const fn from_ip_and_port(ip: IP, port: Port) -> Self {
        match ip {
            IP::IPv4(ip) => Self::IPv4(ip, port),
            IP::IPv6(ip) => Self::IPv6(ip, port),
        }
    }
}

impl From<(IPv4, Port)> for SockerAddress {
    fn from((ip, port): (IPv4, Port)) -> Self {
        Self::IPv4(ip, port)
    }
}

impl From<(IPv6, Port)> for SockerAddress {
    fn from((ip, port): (IPv6, Port)) -> Self {
        Self::IPv6(ip, port)
    }
}

impl From<PathOwned> for SockerAddress {
    fn from(path: PathOwned) -> Self {
        Self::Local(path)
    }
}
