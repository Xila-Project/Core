#![no_std]

extern crate alloc;

mod directory;
mod error;
mod file;
mod file_system;
mod hierarchy;
mod item;
mod r#macro;
mod pipe;
mod socket;
mod synchronous_directory;
mod synchronous_file;

pub use directory::*;
pub use error::*;
use exported_file_system::{Flags, StateFlags};
pub use file::*;
pub use file_system::*;
pub use hierarchy::*;
pub use item::*;
pub use socket::SockerAddress;
pub use synchronous_directory::*;
pub use synchronous_file::*;

pub extern crate file_system as exported_file_system;

use core::{future::poll_fn, task::Poll};

pub async fn poll<O>(mut operation: impl FnMut() -> Result<O>) -> Result<O> {
    poll_fn(|context| match operation() {
        Err(Error::FileSystem(::file_system::Error::RessourceBusy)) | Err(Error::RessourceBusy) => {
            context.waker().wake_by_ref();
            Poll::Pending
        }
        other => core::task::Poll::Ready(other),
    })
    .await
}

pub fn blocking_operation<O>(flags: Flags, mut operation: impl FnMut() -> Result<O>) -> Result<O> {
    let non_blocking = flags.get_status().contains(StateFlags::NonBlocking);

    loop {
        match operation() {
            Err(Error::FileSystem(::file_system::Error::RessourceBusy))
            | Err(Error::RessourceBusy) => {
                if non_blocking {
                    return Err(Error::FileSystem(::file_system::Error::RessourceBusy));
                }
            }
            other => return other,
        }
    }
}
