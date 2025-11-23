use std::fs;

pub static SHORTCUT: &str = r#"
{
    "name": "Calculator",
    "command": "/binaries/wasm",
    "arguments": ["/binaries/calculator"],
    "terminal": false,
    "icon_string": "Ca",
    "icon_color": [158, 158, 158]
}"#;

pub const SHORTCUT_PATH: &str = "/configuration/shared/shortcuts/calculator.json";

#[unsafe(no_mangle)]
pub extern "C" fn __install() {
    println!("Installing Calculator shortcut...");

    fs::write(SHORTCUT_PATH, SHORTCUT).unwrap();

    println!("Calculator shortcut installed.");
}
