use std::{path::Path, process::Command};
use syn;

pub fn format_rust(file_path: &Path) -> Result<(), String> {
    Command::new("rustfmt")
        .arg(
            file_path
                .to_str()
                .ok_or("Error converting path to string")?,
        )
        .status()
        .map_err(|code| format!("Error running rustfmt : {code}"))?;

    Ok(())
}

pub fn format_c(file_path: &Path) -> Result<(), String> {
    Command::new("clang-format")
        .arg("-i")
        .arg(
            file_path
                .to_str()
                .ok_or("Error converting path to string")?,
        )
        .status()
        .map_err(|code| format!("Error running clang-format : {code}"))?;

    Ok(())
}

const NAMES: [(&str, &str); 1] = [("obj", "object")];

pub fn format_identifier(prefix: &str, function_name: &str) -> String {
    // - Remove prefix
    let function_name = if function_name.starts_with("lv_") {
        function_name.replacen("lv_", "", 1)
    } else {
        function_name.to_string()
    };

    let function_name = if !function_name.starts_with(prefix) {
        format!("{prefix}{function_name}")
    } else {
        function_name
    };

    // - Replace names

    function_name
        .split("_")
        .map(|part| match NAMES.iter().find(|(old, _)| *old == part) {
            Some((_, new)) => new.to_string(),
            None => part.to_string(),
        })
        .collect::<Vec<String>>()
        .join("_")
}

pub fn snake_to_upper_camel_case(input: &str) -> String {
    input
        .split('_')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

pub fn snake_ident_to_upper_camel(ident: &syn::Ident) -> syn::Ident {
    let name = ident.to_string();
    let camel_case = snake_to_upper_camel_case(&name);
    syn::Ident::new(&camel_case, ident.span())
}
