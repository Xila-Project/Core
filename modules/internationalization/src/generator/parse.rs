use std::fs;
use std::path::Path;

pub fn read_and_parse_locale_file(path: impl AsRef<Path>) -> Result<toml::Value, String> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let toml_path = Path::new(&manifest_dir).join(&path);

    let toml_content = fs::read_to_string(&toml_path)
        .map_err(|err| format!("Failed to read locale file at {:?}: {}", toml_path, err))?;

    let toml_value = toml::from_str(&toml_content)
        .map_err(|err| format!("Failed to parse TOML content from {:?}: {}", toml_path, err))?;

    Ok(toml_value)
}
