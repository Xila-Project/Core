#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub type User_identifier_type = u16;
pub type Group_identifier_type = u16;
pub const Root_user_identifier: User_identifier_type = 0;
pub const Root_group_identifier: Group_identifier_type = 0;

mod Manager;
pub use Manager::*;

mod Error;
pub use Error::*;
