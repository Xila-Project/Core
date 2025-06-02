use embassy_executor::{raw, Spawner};
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::sync::{Condvar, Mutex};

/// Single-threaded std-based executor.
pub struct Executor_type {
    Inner: raw::Executor,
    Not_send: PhantomData<*mut ()>,
    Signaler: &'static Signaler_type,
    Stop: AtomicBool,
}

impl Executor_type {
    /// Create a new Executor.
    pub fn New() -> Self {
        let Signaler = Box::leak(Box::new(Signaler_type::New()));
        Self {
            Inner: raw::Executor::new(Signaler as *mut Signaler_type as *mut ()),
            Not_send: PhantomData,
            Signaler,
            Stop: AtomicBool::new(false),
        }
    }

    pub fn Stop(&self) {
        self.Stop.store(true, std::sync::atomic::Ordering::SeqCst);
        self.Signaler.Signal();
    }

    /// Get a spawner for this executor.
    pub fn Spawner(&'static self) -> Spawner {
        self.Inner.spawner()
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
    pub fn Run(&'static mut self, init: impl FnOnce(Spawner)) {
        init(self.Inner.spawner());

        while !self.Stop.load(std::sync::atomic::Ordering::SeqCst) {
            unsafe { self.Inner.poll() };
            self.Signaler.Wait();
        }
    }
}

struct Signaler_type {
    Mutex: Mutex<bool>,
    Condvar: Condvar,
}

impl Signaler_type {
    fn New() -> Self {
        Self {
            Mutex: Mutex::new(false),
            Condvar: Condvar::new(),
        }
    }

    fn Wait(&self) {
        let mut Signaled = self.Mutex.lock().unwrap();
        while !*Signaled {
            Signaled = self.Condvar.wait(Signaled).unwrap();
        }
        *Signaled = false;
    }

    fn Signal(&self) {
        let mut Signaled = self.Mutex.lock().unwrap();
        *Signaled = true;
        self.Condvar.notify_one();
    }
}
