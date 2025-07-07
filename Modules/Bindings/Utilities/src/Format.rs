use std::{path::Path, process::Command};

pub fn Format_rust(File_path: &Path) -> Result<(), String> {
    Command::new("rustfmt")
        .arg(
            File_path
                .to_str()
                .ok_or("Error converting path to string")?,
        )
        .status()
        .map_err(|code| format!("Error running rustfmt : {code}"))?;

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
        .map_err(|code| format!("Error running clang-format : {code}"))?;

    Ok(())
}

const NAMES: [(&str, &str); 1] = [("obj", "object")];

pub fn Format_identifier(Prefix: &str, Function_name: &str) -> String {
    // - Remove prefix
    let Function_name = if Function_name.starts_with("lv_") {
        Function_name.replacen("lv_", "", 1)
    } else {
        Function_name.to_string()
    };

    let Function_name = if !Function_name.starts_with(Prefix) {
        format!("{Prefix}{Function_name}")
    } else {
        Function_name
    };

    // - Replace names
    let Function_name = Function_name
        .split("_")
        .map(|part| match NAMES.iter().find(|(Old, _)| *Old == part) {
            Some((_, new)) => new.to_string(),
            None => part.to_string(),
        })
        .collect::<Vec<String>>()
        .join("_");

    // - Make first letter uppercase
    // let mut Chars = Function_name.chars();

    // let Function_name = match Chars.next() {
    //     None => String::new(),
    //     Some(first_char) => {
    //         first_char.to_uppercase().collect::<String>() + &Chars.as_str().to_lowercase()
    //     }
    // };

    Function_name
}
