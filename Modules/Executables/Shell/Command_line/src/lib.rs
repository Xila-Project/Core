#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use Executable::Standard_type;

mod Commands;
mod Device;
mod Error;
mod Main;
mod Parser;
mod Resolver;
mod Tokenizer;

pub use Device::*;

use Error::*;
use File_system::Path_owned_type;
pub struct Shell_type {
    Standard: Standard_type,
    Current_directory: Path_owned_type,
    Running: bool,
    User: String,
    Host: String,
}
