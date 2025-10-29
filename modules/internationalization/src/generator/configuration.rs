use std::path::{Path, PathBuf};

pub struct Configuration {
    pub input_path: PathBuf,
    pub locale: String,
    pub fallback: String,
    pub output_path: PathBuf,
}

fn get_locale_from_environment_variable() -> String {
    std::env::var("INTERNATIONALIZATION_LOCALE").unwrap_or("en".to_string())
}

impl Default for Configuration {
    fn default() -> Self {
        let manifest_path =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let manifest_dir = Path::new(&manifest_path);

        let output_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
        let output_path = Path::new(&output_dir);

        Configuration {
            input_path: manifest_dir.join("src").join("locales.toml"),
            locale: get_locale_from_environment_variable(),
            fallback: "en".to_string(),
            output_path: output_path.join("internationalization.rs"),
        }
    }
}
