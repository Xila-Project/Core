#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate alloc;

use alloc::boxed::Box;
use Desk::Desk_type;
use Executable::Standard_type;
use Home::Home_type;
use Layout::Layout_type;
use Login::Login_type;

mod Desk;
mod Error;
mod Home;
mod Icon;
mod Layout;
mod Login;
mod Main;
mod Shortcut;

pub struct Shell_type {
    _Standard: Standard_type,
    Running: bool,
    Layout: Layout_type,
    Desk: Option<Box<Desk_type>>,
    _Home: Option<Box<Home_type>>,
    Login: Option<Box<Login_type>>,
}

pub struct Shell_executable_type;

Executable::Implement_executable_device!(
    Structure: Shell_executable_type,
    Mount_path: "/Binaries/Graphical_shell",
    Main_function: Main::Main,
);
