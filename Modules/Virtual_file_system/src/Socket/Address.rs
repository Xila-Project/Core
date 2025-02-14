use File_system::Path_owned_type;

use Network::{IP_type, IPv4_type, IPv6_type, Port_type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Socket_address_type {
    IPv4(IPv4_type, Port_type),
    IPv6(IPv6_type, Port_type),
    Local(Path_owned_type),
}

impl Socket_address_type {
    pub fn Into_IP_and_port(self) -> Option<(IP_type, Port_type)> {
        match self {
            Self::IPv4(IP, Port) => Some((IP.into(), Port)),
            Self::IPv6(IP, Port) => Some((IP.into(), Port)),
            _ => None,
        }
    }

    pub const fn From_IP_and_port(IP: IP_type, Port: Port_type) -> Self {
        match IP {
            IP_type::IPv4(IP) => Self::IPv4(IP, Port),
            IP_type::IPv6(IP) => Self::IPv6(IP, Port),
        }
    }
}

impl From<(IPv4_type, Port_type)> for Socket_address_type {
    fn from((IP, Port): (IPv4_type, Port_type)) -> Self {
        Self::IPv4(IP, Port)
    }
}

impl From<(IPv6_type, Port_type)> for Socket_address_type {
    fn from((IP, Port): (IPv6_type, Port_type)) -> Self {
        Self::IPv6(IP, Port)
    }
}

impl From<Path_owned_type> for Socket_address_type {
    fn from(Path: Path_owned_type) -> Self {
        Self::Local(Path)
    }
}
