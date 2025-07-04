#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

extern crate alloc;

#[cfg(any(test, feature = "std"))]
extern crate std;

mod Environment_variable;
mod Error;
mod Join_handle;
mod Manager;
mod Signal;
mod Task;

pub use Environment_variable::*;
pub use Error::*;
pub use Futures;
pub use Join_handle::*;
pub use Manager::*;
pub use Signal::*;
pub use Task::*;
pub use Task_macros::{Run_with_executor, Test};

pub use embassy_executor;
