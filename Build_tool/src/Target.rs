#[derive(Debug, Clone, Copy, Default)]
pub enum Target_type {
    Linux,
    Windows,
    ESP32,
    ESP32_S3,
    #[default]
    Native,
}

impl TryFrom<&str> for Target_type {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "linux" => Ok(Target_type::Linux),
            "windows" => Ok(Target_type::Windows),
            "esp32" => Ok(Target_type::ESP32),
            "esp32s3" => Ok(Target_type::ESP32_S3),
            "native" => Ok(Target_type::Native),
            _ => Err(format!("Unknown target type : {}", s)),
        }
    }
}

impl Target_type {
    pub fn Get_toolchain(&self) -> String {
        match self {
            Target_type::ESP32 | Target_type::ESP32_S3 => "+esp".to_string(),
            _ => "+stable".to_string(),
        }
    }

    pub fn Get_environment_variables(&self) -> Vec<(String, String)> {
        match self {
            Target_type::ESP32 => vec![("MCU".to_string(), "esp32".to_string())],
            Target_type::ESP32_S3 => vec![("MCU".to_string(), "esp32s3".to_string())],
            _ => vec![],
        }
    }

    pub fn Get_arguments(&self) -> Vec<String> {
        match self {
            Target_type::ESP32 => vec![
                "--target".to_string(),
                "xtensa-esp32-espidf".to_string(),
                "-Z".to_string(),
                "build-std=std,panic_abort".to_string(),
            ],
            Target_type::ESP32_S3 => vec![
                "--target".to_string(),
                "xtensa-esp32s3-espidf".to_string(),
                "-Z".to_string(),
                "build-std=std,panic_abort".to_string(),
            ],
            Target_type::Linux => vec![
                "--target".to_string(),
                "x86_64-unknown-linux-gnu".to_string(),
            ],
            Target_type::Windows => {
                vec!["--target".to_string(), "x86_64-pc-windows-gnu".to_string()]
            }
            Target_type::Native => vec![],
        }
    }
}
