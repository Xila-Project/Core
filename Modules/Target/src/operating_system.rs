use std::env;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operating_system_type {
    Windows,
    Linux,
    MacOS,
    ESP_IDF,
    WASI,
}

impl Operating_system_type {
    pub fn get() -> Operating_system_type {
        Operating_system_type::from(Self::get_raw())
    }

    pub fn get_raw() -> String {
        env::var("CARGO_CFG_TARGET_OS").unwrap()
    }
}

impl From<String> for Operating_system_type {
    fn from(s: String) -> Self {
        match s.as_str() {
            "windows" => Operating_system_type::Windows,
            "linux" => Operating_system_type::Linux,
            "macos" => Operating_system_type::MacOS,
            "espidf" => Operating_system_type::ESP_IDF,
            "wasi" => Operating_system_type::WASI,
            _ => panic!("Unknown operating system type : {s}"),
        }
    }
}
