use alloc::sync::Arc;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

pub struct Join_handle_type<T>(Arc<Signal<CriticalSectionRawMutex, T>>);

unsafe impl<T> Send for Join_handle_type<T> {}
unsafe impl<T> Sync for Join_handle_type<T> {}

impl<T> Join_handle_type<T> {
    pub fn new() -> (Self, Self) {
        let signal = Signal::<CriticalSectionRawMutex, T>::new();

        let arc = Arc::new(signal);

        (Self(arc.clone()), Self(arc))
    }

    pub(crate) fn signal(&self, value: T) {
        self.0.signal(value);
    }

    pub async fn join(self) -> T {
        self.0.wait().await
    }
}
