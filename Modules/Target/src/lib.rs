#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

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
    Architecture: Architecture_type,
    Operating_system: Operating_system_type,
    Family: Family_type,
    Vendor: Vendor_type,
}

impl Target_type {
    pub fn Get_architecture(&self) -> Architecture_type {
        self.Architecture
    }

    pub fn Get_operating_system(&self) -> Operating_system_type {
        self.Operating_system
    }

    pub fn Get_family(&self) -> Family_type {
        self.Family
    }

    pub fn Get_current() -> Target_type {
        Target_type {
            Architecture: Architecture_type::from(env::var("CARGO_CFG_TARGET_ARCH").unwrap()),
            Operating_system: Operating_system_type::from(env::var("CARGO_CFG_TARGET_OS").unwrap()),
            Family: Family_type::from(env::var("CARGO_CFG_TARGET_FAMILY").unwrap()),
            Vendor: Vendor_type::from(env::var("CARGO_CFG_TARGET_VENDOR").unwrap()),
        }
    }
}
