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

use core::time::Duration;
use embassy_time::Timer;

pub use embassy_executor;
pub use environment_variable::*;
pub use error::*;
pub use futures;
pub use futures::yield_now;
pub use join_handle::*;
pub use manager::*;
pub use signal::*;
pub use task::*;
pub use task_macros::{run, test};

/// Sleep the current thread for a given duration.
pub async fn sleep(duration: Duration) {
    let nano_seconds = duration.as_nanos();

    Timer::after(embassy_time::Duration::from_nanos(nano_seconds as u64)).await
}
