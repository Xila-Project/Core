#![allow(non_camel_case_types)]

pub type User_identifier_type = u16;
pub type Group_identifier_type = u16;

mod Manager;
pub use Manager::*;

mod User;
pub use User::*;
