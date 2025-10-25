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

#[unsafe(no_mangle)]
pub extern "C" fn __install() {
    fs::write("/configuration/shared/shortcuts/calculator.json", SHORTCUT).unwrap();
}
