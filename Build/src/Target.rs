#![allow(dead_code)]

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Architecture_type {
    x86,
    x86_64,
    Arm,
    Aarch64,
    Xtensa,
    RiscV,
}

impl From<String> for Architecture_type {
    fn from(s: String) -> Self {
        match s.as_str() {
            "x86" => Architecture_type::x86,
            "x86_64" => Architecture_type::x86_64,
            "arm" => Architecture_type::Arm,
            "aarch64" => Architecture_type::Aarch64,
            "xtensa" => Architecture_type::Xtensa,
            "riscv" => Architecture_type::RiscV,
            _ => panic!("Unknown architecture type : {}", s),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operating_system_type {
    Windows,
    Linux,
    MacOS,
    ESP_IDF,
}

impl From<String> for Operating_system_type {
    fn from(s: String) -> Self {
        match s.as_str() {
            "windows" => Operating_system_type::Windows,
            "linux" => Operating_system_type::Linux,
            "macos" => Operating_system_type::MacOS,
            "espidf" => Operating_system_type::ESP_IDF,
            _ => panic!("Unknown operating system type : {}", s),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Family_type {
    Unix,
    Windows,
    Wasm,
}

impl From<String> for Family_type {
    fn from(s: String) -> Self {
        match s.as_str() {
            "unix" => Family_type::Unix,
            "windows" => Family_type::Windows,
            "wasm" => Family_type::Wasm,
            _ => panic!("Unknown family type : {}", s),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Vendor_type {
    Espressif,
    Unknown,
}

impl From<String> for Vendor_type {
    fn from(s: String) -> Self {
        match s.as_str() {
            "espressif" => Vendor_type::Espressif,
            _ => Vendor_type::Unknown,
        }
    }
}

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
            Architecture: Architecture_type::from(std::env::var("CARGO_CFG_TARGET_ARCH").unwrap()),
            Operating_system: Operating_system_type::from(
                std::env::var("CARGO_CFG_TARGET_OS").unwrap(),
            ),
            Family: Family_type::from(std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap()),
            Vendor: Vendor_type::from(std::env::var("CARGO_CFG_TARGET_VENDOR").unwrap()),
        }
    }
}
