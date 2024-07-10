#[derive(Debug, Clone, Copy)]
pub enum Command_type {
    Build,
    Clean,
    Run,
    Test,
    Format,
    Doc,
    Clippy,
    Check,
}

impl TryFrom<&str> for Command_type {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "build" => Ok(Command_type::Build),
            "clean" => Ok(Command_type::Clean),
            "run" => Ok(Command_type::Run),
            "test" => Ok(Command_type::Test),
            "format" => Ok(Command_type::Format),
            "doc" => Ok(Command_type::Doc),
            "clippy" => Ok(Command_type::Clippy),
            "check" => Ok(Command_type::Check),
            _ => Err(format!("Unknown command : {}", s)),
        }
    }
}

impl Command_type {
    pub fn Get_cargo_command(&self) -> String {
        match self {
            Command_type::Build => "build".to_string(),
            Command_type::Clean => "clean".to_string(),
            Command_type::Run => "run".to_string(),
            Command_type::Test => "test".to_string(),
            Command_type::Format => "fmt".to_string(),
            Command_type::Doc => "doc".to_string(),
            Command_type::Clippy => "clippy".to_string(),
            Command_type::Check => "check".to_string(),
        }
    }

    pub fn Is_target_needed(&self) -> bool {
        match self {
            Command_type::Clean | Command_type::Format | Command_type::Doc => false,
            Command_type::Build
            | Command_type::Run
            | Command_type::Test
            | Command_type::Clippy
            | Command_type::Check => true,
        }
    }
}
