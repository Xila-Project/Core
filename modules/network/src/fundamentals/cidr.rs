use embassy_net::Ipv4Cidr;

use crate::{IPv4, IPv6};

#[repr(C)]
pub struct CIDR<IP> {
    pub ip: IP,
    pub prefix_length: u8,
}

impl<IP> CIDR<IP> {
    pub const fn new(ip: IP, prefix_length: u8) -> Self {
        Self { ip, prefix_length }
    }
}

impl CIDR<IPv4> {
    pub const fn from_embassy_cidr(value: Ipv4Cidr) -> Self {
        Self {
            ip: IPv4::from_embassy(value.address()),
            prefix_length: value.prefix_len(),
        }
    }

    pub const fn to_embassy_cidr(&self) -> Ipv4Cidr {
        Ipv4Cidr::new(self.ip.into_embassy(), self.prefix_length)
    }
}

impl CIDR<IPv6> {
    pub const fn from_embassy_cidr(value: embassy_net::Ipv6Cidr) -> Self {
        Self {
            ip: IPv6::from_embassy(value.address()),
            prefix_length: value.prefix_len(),
        }
    }

    pub const fn to_embassy_cidr(&self) -> embassy_net::Ipv6Cidr {
        embassy_net::Ipv6Cidr::new(self.ip.into_embassy(), self.prefix_length)
    }
}

pub type CidrIPv4 = CIDR<IPv4>;
pub type CidrIPv6 = CIDR<IPv6>;
