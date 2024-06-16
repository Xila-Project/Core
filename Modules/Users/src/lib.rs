#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub type User_identifier_type = u16;
pub type Group_identifier_type = u16;

mod Manager;
pub use Manager::*;

mod User;
pub use User::*;

mod Error;
pub use Error::*;
