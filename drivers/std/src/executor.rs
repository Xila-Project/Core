use embassy_executor::{Spawner, raw};
use std::boxed::Box;
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::sync::{Condvar, Mutex};
use synchronization::blocking_mutex::raw::CriticalSectionRawMutex;
use synchronization::signal::Signal;
use task::SpawnerIdentifier;

/// Single-threaded std-based executor.
pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
    signaler: &'static Signaler,
    stop: AtomicBool,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    /// Create a new Executor.
    pub fn new() -> Self {
        let signaler = Box::leak(Box::new(Signaler::new()));
        Self {
            inner: raw::Executor::new(signaler as *mut Signaler as *mut ()),
            not_send: PhantomData,
            signaler,
            stop: AtomicBool::new(false),
        }
    }

    pub fn stop(&self) {
        self.stop.store(true, std::sync::atomic::Ordering::SeqCst);
        self.signaler.signal();
    }

    /// Get a spawner for this executor.
    pub fn spawner(&'static self) -> Spawner {
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
    pub fn run(&'static self, init: impl FnOnce(Spawner, &'static Self)) {
        init(self.inner.spawner(), self);

        while !self.stop.load(std::sync::atomic::Ordering::SeqCst) {
            unsafe { self.inner.poll() };
            self.signaler.wait();
        }
    }

    pub fn start(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        init(self.inner.spawner());

        loop {
            unsafe { self.inner.poll() };
            self.signaler.wait()
        }
    }
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

#[macro_export]
macro_rules! instantiate_static_executor {
    () => {{
        static mut __EXECUTOR: Option<$crate::executor::Executor> = None;

        unsafe {
            if __EXECUTOR.is_none() {
                __EXECUTOR = Some($crate::executor::Executor::new());
            }
            __EXECUTOR.as_mut().expect("Executor is not initialized")
        }
    }};
}

pub use instantiate_static_executor;

pub async fn new_thread_executor() -> SpawnerIdentifier {
    let task_manager = task::get_instance();

    // Create a new OnceLock for each call to allow multiple thread executors
    let signal: &'static Signal<CriticalSectionRawMutex, SpawnerIdentifier> =
        Box::leak(Box::new(Signal::new()));

    std::thread::spawn(move || {
        // Use Box::leak to create a 'static reference for this thread's executor
        let executor = Box::leak(Box::new(Executor::new()));

        executor.start(move |spawner: Spawner| {
            let spawner_id = task_manager.register_spawner(spawner).unwrap();

            signal.signal(spawner_id);
        });
    });

    let spawner_id = signal.wait().await;

    unsafe {
        let _ = Box::from_raw(
            signal as *const _ as *mut Signal<CriticalSectionRawMutex, SpawnerIdentifier>,
        );
        // Clean up the leaked Box to avoid memory leak
    }

    spawner_id
}
