#![allow(non_camel_case_types)]

mod Architecture;
mod Family;
mod Operating_system;
mod Vendor;

use std::env;

pub use Architecture::*;
pub use Family::*;
pub use Operating_system::*;
pub use Vendor::*;

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

    pub fn Get_operating_system(&self) -> Operating_system_type {
        self.operating_system
    }

    pub fn Get_family(&self) -> Family_type {
        self.family
    }

    pub fn Get_current() -> Target_type {
        Target_type {
            architecture: Architecture_type::from(env::var("CARGO_CFG_TARGET_ARCH").unwrap()),
            operating_system: Operating_system_type::from(env::var("CARGO_CFG_TARGET_OS").unwrap()),
            family: Family_type::from(env::var("CARGO_CFG_TARGET_FAMILY").unwrap()),
            vendor: Vendor_type::from(env::var("CARGO_CFG_TARGET_VENDOR").unwrap()),
        }
    }
}
