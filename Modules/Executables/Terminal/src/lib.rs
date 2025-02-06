#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

mod Device;
mod Executable;
mod Main;
mod Terminal;

mod Error;

pub use Executable::*;

pub const Shortcut: &str = r#"
{
    "Name": "Terminal",
    "Command": "/Binaries/Terminal",
    "Arguments": "",
    "Terminal": false,
    "Icon_string": ">_"
}"#;
