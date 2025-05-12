use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use Synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, Arc};

pub struct Join_handle_type<T>(Arc<Signal<CriticalSectionRawMutex, T>>);

impl<T> Join_handle_type<T> {
    pub fn New() -> (Self, Self) {
        let Signal = Signal::<CriticalSectionRawMutex, T>::new();

        let Arc = Arc::new(Signal);

        (Self(Arc.clone()), Self(Arc))
    }

    pub(crate) fn Signal(&self, Value: T) {
        self.0.signal(Value);
    }

    pub async fn Join(self) -> T {
        self.0.wait().await
    }
}
