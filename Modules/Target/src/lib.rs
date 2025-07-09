#![allow(non_camel_case_types)]

mod architecture;
mod family;
mod operating_system;
mod vendor;

use std::env;

pub use architecture::*;
pub use family::*;
pub use operating_system::*;
pub use vendor::*;

#[derive(Debug, PartialEq)]
pub struct Target_type {
    architecture: Architecture_type,
    operating_system: Operating_system_type,
    family: Family_type,
    vendor: Vendor_type,
}

impl Target_type {
    pub fn get_architecture(&self) -> Architecture_type {
        self.architecture
    }

    pub fn get_operating_system(&self) -> Operating_system_type {
        self.operating_system
    }

    pub fn get_family(&self) -> Family_type {
        self.family
    }

    pub fn get_current() -> Target_type {
        Target_type {
            architecture: Architecture_type::from(env::var("CARGO_CFG_TARGET_ARCH").unwrap()),
            operating_system: Operating_system_type::from(env::var("CARGO_CFG_TARGET_OS").unwrap()),
            family: Family_type::from(env::var("CARGO_CFG_TARGET_FAMILY").unwrap()),
            vendor: Vendor_type::from(env::var("CARGO_CFG_TARGET_VENDOR").unwrap()),
        }
    }
}
