#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use Desk::Desk_type;
use Executable::Standard_type;
use Home::Home_type;
use Layout::Layout_type;

mod Desk;
mod Device;
mod Error;
mod Home;
mod Icon;
mod Layout;
mod Main;
mod Terminal;

pub use Device::*;
use Terminal::Terminal_type;

pub struct Shell_type {
    Standard: Standard_type,
    User: String,
    Running: bool,
    Layout: Layout_type,
    Desk: Desk_type,
    Home: Home_type,
    Terminal: Option<Terminal_type>,
}
