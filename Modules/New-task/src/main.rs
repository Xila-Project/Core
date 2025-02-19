#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::{
    ptr::NonNull,
    sync::{Condvar, Mutex, RwLock},
    task::{Context, Poll},
};

use core::future::Future;

use embassy_executor::{
    raw::{self, Executor, TaskRef},
    Spawner,
};
use embassy_sync::once_lock::OnceLock;
use embassy_time::{Instant, Timer};
use futures::{
    executor::{self, block_on},
    FutureExt,
};
use log::*;

use core::future::poll_fn;

fn get_task_pointer_sync() -> usize {
    block_on(get_task_pointer())
}

fn sub_task_1() {
    info!("sub timer 1 at : {:?}", Instant::now());

    block_on(test_fn());
}

async fn test_fn() {
    info!("test fn at : {:?}", Instant::now());
}

// UNSAFE: A static mutable variable holding a pointer to a Context.
// This is only for demonstration purposes and is unsound!
static mut SAVED_CONTEXT: Option<*mut Context<'static>> = None;

/// Saves the current async context into a static variable.
/// WARNING: This converts the lifetime to 'static unsafely.
fn save_context(cx: &mut Context<'_>) {
    unsafe {
        SAVED_CONTEXT = Some(core::mem::transmute(cx));
    }
}

/// Retrieves the saved context, if any.
/// WARNING: The returned reference has a 'static lifetime, which is unsound.
fn get_saved_context() -> Option<&'static mut Context<'static>> {
    unsafe { SAVED_CONTEXT.map(|ptr| &mut *ptr) }
}

fn big_array_function(Context: &mut Context<'_>) {
    let mut _big_array = [0u8; 1024 * 1024 * 2];

    _big_array.iter_mut().for_each(|x| *x = 0);
}

fn wait_sync(Context: &mut Context<'_>) -> core::task::Poll<()> {
    let mut timer_future = Box::pin(Timer::after_secs(1));

    timer_future.as_mut().poll(Context)
}

#[embassy_executor::task]
async fn task_1() {
    loop {
        // poll_fn(|cx| big_array_function(cx));

        info!("timer 1 at : {:?}", Instant::now());
        Timer::after_secs(1).await;
        info!("tick : {:x?}", get_task_pointer().await);

        sub_task_1();
    }
}

#[embassy_executor::task]
async fn task_2() {
    loop {
        info!("timer 2 at : {:?}", Instant::now());
        Timer::after_secs(1).await;
        info!("tock : {:x?}", get_task_pointer().await);
    }
}

#[embassy_executor::task]
async fn task_start() {
    let task = get_task_pointer().await;
}

async fn get_task_pointer() -> usize {
    poll_fn(|cx| unsafe {
        let task = raw::task_from_waker(cx.waker());

        let inner: NonNull<u8> = core::mem::transmute(task);

        let inner = inner.as_ptr() as usize;

        Poll::Ready(inner)
    })
    .await
}

struct Signaler {
    mutex: Mutex<bool>,
    condvar: Condvar,
}

impl Signaler {
    fn new() -> Self {
        Self {
            mutex: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    fn wait(&self) {
        let mut signaled = self.mutex.lock().unwrap();
        while !*signaled {
            signaled = self.condvar.wait(signaled).unwrap();
        }
        *signaled = false;
    }

    fn signal(&self) {
        let mut signaled = self.mutex.lock().unwrap();
        *signaled = true;
        self.condvar.notify_one();
    }
}

struct Executor_type {
    inner: raw::Executor,
    signaler: Signaler,
    current_task: RwLock<Option<usize>>,
}

impl Executor_type {}

static Low_executor: OnceLock<Executor> = OnceLock::new();
static Signaler: OnceLock<Signaler> = OnceLock::new();

#[export_name = "__pender"]
fn __pender(context: *mut Signaler) {
    let signaler: &'static Signaler = unsafe { &*context };
    signaler.signal()
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_nanos()
        .init();

    let s = Signaler::new();

    let s = Signaler.get_or_init(|| s);

    let e = Executor::new(s as *const _ as *mut _);

    let E = Low_executor.get_or_init(|| e);

    let spawner = E.spawner();

    spawner.spawn(task_1()).unwrap();
    spawner.spawn(task_2()).unwrap();

    loop {
        unsafe {
            E.poll();
            s.wait();
        }
    }
}
