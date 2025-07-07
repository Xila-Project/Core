use embassy_executor::{raw, Spawner};
use std::boxed::Box;
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::sync::{Condvar, Mutex};

/// Single-threaded std-based executor.
pub struct Executor_type {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
    signaler: &'static Signaler_type,
    stop: AtomicBool,
}

impl Executor_type {
    /// Create a new Executor.
    pub fn New() -> Self {
        let signaler = Box::leak(Box::new(Signaler_type::new()));
        Self {
            inner: raw::Executor::new(signaler as *mut Signaler_type as *mut ()),
            not_send: PhantomData,
            signaler,
            stop: AtomicBool::new(false),
        }
    }

    pub fn Stop(&self) {
        self.stop.store(true, std::sync::atomic::Ordering::SeqCst);
        self.signaler.Signal();
    }

    /// Get a spawner for this executor.
    pub fn Spawner(&'static self) -> Spawner {
        self.inner.spawner()
    }

    /// Run the executor.
    ///
    /// The `init` closure is called with a [`Spawner`] that spawns tasks on
    /// this executor. Use it to spawn the initial task(s). After `init` returns,
    /// the executor starts running the tasks.
    ///
    /// To spawn more tasks later, you may keep copies of the [`Spawner`] (it is `Copy`),
    /// for example by passing it as an argument to the initial tasks.
    ///
    /// This function requires `&'static mut self`. This means you have to store the
    /// Executor instance in a place where it'll live forever and grants you mutable
    /// access. There's a few ways to do this:
    ///
    /// - a [StaticCell](https://docs.rs/static_cell/latest/static_cell/) (safe)
    /// - a `static mut` (unsafe)
    /// - a local variable in a function you know never returns (like `fn main() -> !`), upgrading its lifetime with `transmute`. (unsafe)
    ///
    /// This function never returns.
    pub fn Run(&'static self, init: impl FnOnce(Spawner, &'static Self)) {
        init(self.inner.spawner(), self);

        while !self.stop.load(std::sync::atomic::Ordering::SeqCst) {
            unsafe { self.inner.poll() };
            self.signaler.Wait();
        }
    }
}

struct Signaler_type {
    mutex: Mutex<bool>,
    condvar: Condvar,
}

impl Signaler_type {
    fn new() -> Self {
        Self {
            mutex: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    fn Wait(&self) {
        let mut signaled = self.mutex.lock().unwrap();
        while !*signaled {
            signaled = self.condvar.wait(signaled).unwrap();
        }
        *signaled = false;
    }

    fn Signal(&self) {
        let mut signaled = self.mutex.lock().unwrap();
        *signaled = true;
        self.condvar.notify_one();
    }
}

#[macro_export]
macro_rules! Instantiate_static_executor {
    () => {{
        static mut __Executor: Option<$crate::Std::Executor::Executor_type> = None;

        unsafe {
            if __Executor.is_none() {
                __Executor = Some($crate::Std::Executor::Executor_type::New());
            }
            __Executor.as_mut().expect("Executor is not initialized")
        }
    }};
}

pub use Instantiate_static_executor;
