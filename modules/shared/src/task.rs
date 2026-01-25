#[macro_export]
macro_rules! poll_ready {
    ($expr:expr) => {
        match $expr {
            Poll::Ready(val) => val,
            Poll::Pending => {
                return Poll::Pending;
            }
        }
    };
}

#[macro_export]
macro_rules! poll_pin_ready {
    ($pin:expr, $context:expr) => {
        $crate::poll_ready!(::core::pin::pin!($pin).poll($context))
    };
}
