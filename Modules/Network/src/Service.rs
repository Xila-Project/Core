use File_system::{Path_owned_type, Path_type};

use crate::{IPv4_type, IPv6_type, Port_type};

pub enum Service_type {
    IPv4((IPv4_type, Port_type)),
    IPv6((IPv6_type, Port_type)),
    Local(Path_owned_type),
}

impl From<(IPv4_type, Port_type)> for Service_type {
    fn from(Value: (IPv4_type, Port_type)) -> Self {
        Self::IPv4(Value)
    }
}

impl From<(IPv6_type, Port_type)> for Service_type {
    fn from(Value: (IPv6_type, Port_type)) -> Self {
        Self::IPv6(Value)
    }
}

impl From<Path_owned_type> for Service_type {
    fn from(Value: Path_owned_type) -> Self {
        Self::Local(Value)
    }
}

impl From<&Path_type> for Service_type {
    fn from(Value: &Path_type) -> Self {
        Self::Local(Value.to_owned())
    }
}
