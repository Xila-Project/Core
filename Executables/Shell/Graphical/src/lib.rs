#![no_std]
#![allow(non_camel_case_types)]

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
    _standard: Standard_type,
    running: bool,
    layout: Layout_type,
    desk: Option<Box<Desk_type>>,
    _home: Option<Box<Home_type>>,
    login: Option<Box<Login_type>>,
}

pub struct Shell_executable_type;

Executable::Implement_executable_device!(
    Structure: Shell_executable_type,
    Mount_path: "/Binaries/Graphical_shell",
    Main_function: Main::Main,
);
