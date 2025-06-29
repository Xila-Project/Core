#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate alloc;

use alloc::string::String;
use Executable::Standard_type;

mod Commands;
mod Error;
mod Main;
mod Parser;
mod Resolver;
mod Tokenizer;

use Error::*;
use File_system::Path_owned_type;
pub struct Shell_type {
    Standard: Standard_type,
    Current_directory: Path_owned_type,
    Running: bool,
    User: String,
    Host: String,
}

pub struct Shell_executable_type;

Executable::Implement_executable_device!(
    Structure: Shell_executable_type,
    Mount_path: "/Binaries/Command_line_shell",
    Main_function: Main::Main,
);
