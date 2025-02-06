#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use Desk::Desk_type;
use Executable::Standard_type;
use Home::Home_type;
use Layout::Layout_type;
use Login::Login_type;

mod Desk;
mod Device;
mod Error;
mod Home;
mod Icon;
mod Layout;
mod Login;
mod Main;
mod Shortcut;

pub use Device::*;

pub struct Shell_type {
    _Standard: Standard_type,
    Running: bool,
    Layout: Layout_type,
    Desk: Option<Desk_type>,
    _Home: Option<Home_type>,
    Login: Option<Login_type>,
}
