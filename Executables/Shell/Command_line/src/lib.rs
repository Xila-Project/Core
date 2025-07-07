#![no_std]
#![allow(non_camel_case_types)]

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
    standard: Standard_type,
    current_directory: Path_owned_type,
    running: bool,
    user: String,
    host: String,
}

pub struct Shell_executable_type;

Executable::Implement_executable_device!(
    Structure: Shell_executable_type,
    Mount_path: "/Binaries/Command_line_shell",
    Main_function: Main::Main,
);
