use std::env;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Family_type {
    Unix,
    Windows,
    Wasm,
}

impl Family_type {
    pub fn get() -> Family_type {
        Family_type::from(Self::Get_raw())
    }

    pub fn Get_raw() -> String {
        env::var("CARGO_CFG_TARGET_FAMILY").unwrap()
    }
}

impl From<String> for Family_type {
    fn from(s: String) -> Self {
        match s.as_str() {
            "unix" => Family_type::Unix,
            "windows" => Family_type::Windows,
            "wasm" => Family_type::Wasm,
            _ => panic!("Unknown family type : {s}"),
        }
    }
}
