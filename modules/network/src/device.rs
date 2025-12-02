use alloc::string::String;
use file_system::{ControlCommand, ControlDirectionFlags};
use heapless::Vec;

use crate::{Duration, IPv4, IPv6, MAXIMUM_HOSTNAME_LENGTH, Port};

#[repr(C)]
pub struct Cidr<T> {
    pub address: T,
    pub prefix_length: u8,
}

impl Cidr<IPv4> {
    pub const fn into_embassy(&self) -> embassy_net::Ipv4Cidr {
        embassy_net::Ipv4Cidr::new(self.address.into_embassy(), self.prefix_length)
    }

    pub const fn from_embassy(value: embassy_net::Ipv4Cidr) -> Self {
        Self {
            address: IPv4::from_embassy(value.address()),
            prefix_length: value.prefix_len(),
        }
    }
}

impl Cidr<IPv6> {
    pub const fn into_embassy(&self) -> embassy_net::Ipv6Cidr {
        embassy_net::Ipv6Cidr::new(self.address.into_embassy(), self.prefix_length)
    }

    pub const fn from_embassy(value: embassy_net::Ipv6Cidr) -> Self {
        Self {
            address: IPv6::from_embassy(value.address()),
            prefix_length: value.prefix_len(),
        }
    }
}

#[repr(C)]
pub struct StaticConfiguration<T> {
    pub ip_address: Cidr<T>,
    pub gateway: Option<T>,
    pub dns_server: [Option<T>; 3],
}

impl StaticConfiguration<IPv4> {
    pub fn into_embassy(self) -> embassy_net::StaticConfigV4 {
        embassy_net::StaticConfigV4 {
            address: self.ip_address.into_embassy(),
            gateway: self.gateway.map(|gateway| gateway.into_embassy()),
            dns_servers: Vec::from_iter(
                self.dns_server
                    .iter()
                    .filter_map(|dns| dns.map(|d| d.into_embassy())),
            ),
        }
    }

    pub fn from_embassy(value: embassy_net::StaticConfigV4) -> Self {
        let mut dns_servers: [Option<IPv4>; 3] = [None, None, None];
        for (i, dns) in value.dns_servers.iter().enumerate().take(3) {
            dns_servers[i] = Some(IPv4::from_embassy(*dns));
        }

        Self {
            ip_address: Cidr::<IPv4>::from_embassy(value.address),
            gateway: value.gateway.map(|gateway| IPv4::from_embassy(gateway)),
            dns_server: dns_servers,
        }
    }
}

impl StaticConfiguration<IPv6> {
    pub fn into_embassy(self) -> embassy_net::StaticConfigV6 {
        embassy_net::StaticConfigV6 {
            address: self.ip_address.into_embassy(),
            gateway: self.gateway.map(|gateway| gateway.into_embassy()),
            dns_servers: Vec::from_iter(
                self.dns_server
                    .iter()
                    .filter_map(|dns| dns.map(|d| d.into_embassy())),
            ),
        }
    }

    pub fn from_embassy(value: embassy_net::StaticConfigV6) -> Self {
        let mut dns_servers: [Option<IPv6>; 3] = [None, None, None];
        for (i, dns) in value.dns_servers.iter().enumerate().take(3) {
            dns_servers[i] = Some(IPv6::from_embassy(*dns));
        }

        Self {
            ip_address: Cidr::<IPv6>::from_embassy(value.address),
            gateway: value.gateway.map(|gateway| IPv6::from_embassy(gateway)),
            dns_server: dns_servers,
        }
    }
}

pub type StaticIpv4Configuration = StaticConfiguration<IPv4>;
pub type StaticIpv6Configuration = StaticConfiguration<IPv6>;

#[repr(C)]
pub struct DhcpConfiguration {
    pub maximum_lease_time: Option<Duration>,

    pub discover_timeout: Duration,
    pub initial_request_timeout: Duration,
    pub request_retries: u16,
    pub minimum_renew_timeout: Duration,
    pub maximum_renew_timeout: Duration,

    pub ignore_naks: bool,
    pub server_port: Port,
    pub client_port: Port,
    pub hostname: Option<[u8; MAXIMUM_HOSTNAME_LENGTH]>,
}

impl DhcpConfiguration {
    pub fn into_embassy(&self) -> embassy_net::DhcpConfig {
        embassy_net::DhcpConfig {
            max_lease_duration: self.maximum_lease_time.map(|d| d.into_embassy()),
            retry_config: smoltcp::socket::dhcpv4::RetryConfig {
                discover_timeout: self.discover_timeout.into_embassy(),
                initial_request_timeout: self.initial_request_timeout.into_embassy(),
                request_retries: self.request_retries,
                min_renew_timeout: self.minimum_renew_timeout.into_embassy(),
                max_renew_timeout: self.maximum_renew_timeout.into_embassy(),
            },
            ignore_naks: self.ignore_naks,
            server_port: self.server_port.into_inner(),
            client_port: self.client_port.into_inner(),
            hostname: Some(heapless::String::new()),
        }
    }
}

impl Default for DhcpConfiguration {
    fn default() -> Self {
        Self {
            maximum_lease_time: None,

            discover_timeout: Duration::from_seconds(10),
            initial_request_timeout: Duration::from_seconds(15),
            request_retries: 5,
            minimum_renew_timeout: Duration::from_seconds(60),
            maximum_renew_timeout: Duration::MAXIMUM,

            ignore_naks: false,
            server_port: Port::DHCP_SERVER,
            client_port: Port::DHCP_CLIENT,
            hostname: [0; MAXIMUM_HOSTNAME_LENGTH],
        }
    }
}

#[repr(C)]
pub enum IpConfiguration<T> {
    Static(T),
    Dhcp(DhcpConfiguration),
}

type Ipv4Configuration = IpConfiguration<StaticIpv4Configuration>;
type Ipv6Configuration = IpConfiguration<StaticIpv6Configuration>;

#[repr(C)]
pub struct WifiConfiguration {
    // TODO: Add fields
}

pub const GET_HARDWARE_ADDRESS: ControlCommand =
    ControlCommand::new::<[u8; 6]>(ControlDirectionFlags::Read, b'N', 1);
pub const IS_LINK_UP: ControlCommand =
    ControlCommand::new::<u8>(ControlDirectionFlags::Read, b'N', 2);
pub const SET_STATE: ControlCommand =
    ControlCommand::new::<bool>(ControlDirectionFlags::Write, b'N', 3);
pub const GET_STATE: ControlCommand =
    ControlCommand::new::<bool>(ControlDirectionFlags::Read, b'N', 4);

pub const GET_IP_V4_CONFIGURATION: ControlCommand =
    ControlCommand::new::<Option<StaticIpv4Configuration>>(ControlDirectionFlags::Read, b'N', 5);
pub const SET_IP_V4_CONFIGURATION: ControlCommand =
    ControlCommand::new::<StaticIpv4Configuration>(ControlDirectionFlags::Write, b'N', 6);

pub const GET_IP_V6_CONFIGURATION: ControlCommand =
    ControlCommand::new::<Option<StaticIpv6Configuration>>(ControlDirectionFlags::Read, b'N', 7);
pub const SET_IP_V6_CONFIGURATION: ControlCommand =
    ControlCommand::new::<StaticIpv6Configuration>(ControlDirectionFlags::Write, b'N', 8);

pub const SET_WIFI_CONFIGURATION: ControlCommand =
    ControlCommand::new::<()>(ControlDirectionFlags::Write, b'N', 9);
pub const GET_WIFI_CONFIGURATION: ControlCommand =
    ControlCommand::new::<WifiConfiguration>(ControlDirectionFlags::Read, b'N', 10);

pub const START_SCAN_WIFI: ControlCommand =
    ControlCommand::new::<()>(ControlDirectionFlags::Write, b'N', 11);
pub const GET_SCAN_RESULTS: ControlCommand =
    ControlCommand::new::<()>(ControlDirectionFlags::Read, b'N', 12);
