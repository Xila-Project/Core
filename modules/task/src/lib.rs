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

use core::{future::poll_fn, pin::Pin, task::Poll, time::Duration};
use embassy_time::Timer;

pub use embassy_executor;
pub use environment_variable::*;
pub use error::*;
pub use join_handle::*;
pub use manager::*;
pub use signal::*;
pub use task::*;
pub use task_macros::{run, test};

/// Sleep the current thread for a given duration.
pub async fn sleep(duration: impl Into<Duration>) {
    let nano_seconds = duration.into().as_nanos();

    Timer::after(embassy_time::Duration::from_nanos(nano_seconds as u64)).await
}

/// Sleeps the task for a duration, breaking early if the TaskManager wakes it up.
pub async fn sleep_interruptible(
    task: TaskIdentifier,
    duration: impl Into<Duration>,
) -> Result<()> {
    let embassy_duration = embassy_time::Duration::from_nanos(duration.into().as_nanos() as u64);

    let mut timer = Timer::after(embassy_duration);
    let mut timer_pinned = Pin::new(&mut timer);

    let mut registered_in_manager = false;
    let mut yielded_once = false;

    poll_fn(move |context| {
        let manager = get_instance();

        if !registered_in_manager {
            match manager.0.try_write() {
                Ok(mut inner) => {
                    if let Ok(task_struct) = Manager::get_task_mutable(&mut inner, task) {
                        task_struct.waker.register(context.waker());
                        registered_in_manager = true;
                    } else {
                        return Poll::Ready(
                            Manager::get_task_mutable(&mut inner, task).map(|_| ()),
                        );
                    }
                }
                Err(_) => {
                    context.waker().wake_by_ref();
                    return Poll::Pending;
                }
            }
        }

        match timer_pinned.as_mut().poll(context) {
            Poll::Ready(()) => Poll::Ready(Ok(())),
            Poll::Pending => {
                if yielded_once {
                    Poll::Ready(Ok(()))
                } else {
                    yielded_once = true;
                    Poll::Pending
                }
            }
        }
    })
    .await
}

/// Suspend the current task indefinitely and register a waker for it,
/// so it can be woken up when the task is sleeping or waiting for a signal.
pub async fn suspend(task: TaskIdentifier) -> Result<()> {
    let manager = get_instance();

    let mut yielded = false;

    poll_fn(|context| {
        if yielded {
            Poll::Ready(Ok(()))
        } else {
            match manager.0.try_write() {
                Ok(mut inner) => {
                    let task = match Manager::get_task_mutable(&mut inner, task) {
                        Ok(t) => t,
                        Err(e) => return Poll::Ready(Err(e)),
                    };

                    task.waker.register(context.waker());

                    yielded = true;
                    Poll::Pending
                }
                Err(_) => {
                    context.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }
    })
    .await
}

#[cfg(target_arch = "wasm32")]
pub async fn yield_now() {
    sleep(Duration::from_millis(10)).await
}
#[cfg(not(target_arch = "wasm32"))]
pub use embassy_futures::yield_now;

pub fn block_on<F: core::future::Future>(future: F) -> F::Output {
    embassy_futures::block_on(future)
}
