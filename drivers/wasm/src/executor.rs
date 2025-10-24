pub use embassy_executor::Executor;

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
