#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Architecture_type {
    x86,
    x86_64,
    Arm,
    Aarch64,
    Xtensa,
    RiscV,
}

impl Architecture_type {
    pub fn Get() -> Architecture_type {
        Architecture_type::from(Self::Get_raw())
    }

    pub fn Get_raw() -> String {
        std::env::var("CARGO_CFG_TARGET_ARCH").unwrap()
    }
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
