#![no_std]

extern crate alloc;

#[cfg(any(test, feature = "std"))]
extern crate std;

mod environment_variable;
mod error;
mod join_handle;
mod manager;
mod signal;
mod task;

pub use environment_variable::*;
pub use error::*;
pub use futures;
pub use join_handle::*;
pub use manager::*;
pub use signal::*;
pub use task::*;
pub use task_macros::{run, test};

pub use embassy_executor;
