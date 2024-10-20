use std::{path::Path, process::Command};

pub fn Format_rust(File_path: &Path) -> Result<(), String> {
    Command::new("rustfmt")
        .arg(
            File_path
                .to_str()
                .ok_or("Error converting path to string")?,
        )
        .status()
        .map_err(|Code| format!("Error running rustfmt : {}", Code))?;

    Ok(())
}

pub fn Format_C(File_path: &Path) -> Result<(), String> {
    Command::new("clang-format")
        .arg("-i")
        .arg(
            File_path
                .to_str()
                .ok_or("Error converting path to string")?,
        )
        .status()
        .map_err(|Code| format!("Error running clang-format : {}", Code))?;

    Ok(())
}

pub fn Format_function_name(Function_name: &str) -> String {
    if Function_name.starts_with("lv_") {
        Function_name.replacen("lv_", "Xila_graphics_", 1)
    } else {
        Function_name.to_string()
    }
}
