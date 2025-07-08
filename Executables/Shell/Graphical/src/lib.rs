#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

use alloc::boxed::Box;
use desk::Desk_type;
use executable::Standard_type;
use home::Home_type;
use layout::Layout_type;
use login::Login_type;

mod desk;
mod error;
mod home;
mod icon;
mod layout;
mod login;
mod main;
mod shortcut;

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
    Main_function: Main::main,
);
