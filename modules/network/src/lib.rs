#![no_std]

extern crate alloc;

mod device;
mod error;
mod fundamentals;
mod manager;
mod protocol;
mod socket;
mod traits;

pub use device::*;
pub use error::*;
pub use fundamentals::*;
pub use manager::*;
pub use protocol::*;
pub use socket::*;
pub use traits::*;

const MAXIMUM_HOSTNAME_LENGTH: usize = 32;
