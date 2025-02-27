pub unsafe trait Raw_rwlock {
    /// Create a new `Raw_rwlock` instance.
    ///
    /// This is a const instead of a method to allow creating instances in const context.
    const INIT: Self;

    /// Lock this `Raw_rwlock`.
    async fn read<R>(&self) -> R;

    /// Lock this `Raw_rwlock`.
    async fn write<W>(&self) -> W;
}

use core::sync::atomic::{AtomicUsize, Ordering};
use embassy_futures::yield_now;
use embassy_sync::waitqueue::{AtomicWaker, WakerRegistration};

const WRITER_BIT: usize = 1 << (usize::BITS - 1);

pub struct RwLock {
    state: AtomicUsize,        // Tracks readers and writer
    writer_waker: AtomicWaker, // Wakes a single waiting writer
}

unsafe impl Raw_rwlock for RwLock {
    const INIT: Self = Self::new();

    async fn read<R>(&self) -> R {
        let guard = self.read();
        let res = yield_now().then(|_| R);
        drop(guard);
        res
    }

    async fn write<R>(&self) -> R {
        let guard = self.write();
        let res = yield_now().then(|_| R);
        drop(guard);
        res
    }
}

impl RwLock {
    pub const fn new() -> Self {
        Self {
            state: AtomicUsize::new(0),
            writer_waker: AtomicWaker::new(),
        }
    }

    pub async fn read(&self) -> AsyncRwLockReadGuard<'_> {
        loop {
            let state = self.state.load(Ordering::Acquire);
            if state & WRITER_BIT == 0 {
                // No writer, try to increment reader count
                if self
                    .state
                    .compare_exchange(state, state + 1, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    return AsyncRwLockReadGuard { lock: self };
                }
            }

            // Yield and retry
            yield_now().await;
        }
    }

    pub async fn write(&self) -> AsyncRwLockWriteGuard<'_> {
        loop {
            let state = self.state.load(Ordering::Acquire);
            if state == 0 {
                // No readers or writer, try to acquire write lock
                if self
                    .state
                    .compare_exchange(0, WRITER_BIT, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    return AsyncRwLockWriteGuard { lock: self };
                }
            }

            // Register the current task's waker and yield
            self.writer_waker.register().await;
            yield_now().await;
        }
    }

    fn release_read(&self) {
        let prev = self.state.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            // Last reader leaving, notify writer
            self.writer_waker.wake();
        }
    }

    fn release_write(&self) {
        self.state.store(0, Ordering::Release);

        // Readers will retry in their loop naturally
        self.writer_waker.wake();
    }
}
