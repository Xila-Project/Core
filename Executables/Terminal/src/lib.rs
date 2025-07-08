#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

mod device;
mod executable;
mod main;
mod terminal;

mod error;

pub use executable::*;

pub const SHORTCUT: &str = r#"
{
    "Name": "Terminal",
    "Command": "/Binaries/Terminal",
    "Arguments": "",
    "Terminal": false,
    "Icon_string": ">_",
    "Icon_color": [0, 0, 0]
}"#;
