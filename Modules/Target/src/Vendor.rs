#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Vendor_type {
    Espressif,
    Unknown,
}

impl Vendor_type {
    pub fn Get() -> Vendor_type {
        Vendor_type::from(Self::Get_raw())
    }

    pub fn Get_raw() -> String {
        std::env::var("CARGO_CFG_TARGET_VENDOR").unwrap()
    }
}

impl From<String> for Vendor_type {
    fn from(s: String) -> Self {
        match s.as_str() {
            "espressif" => Vendor_type::Espressif,
            _ => Vendor_type::Unknown,
        }
    }
}
