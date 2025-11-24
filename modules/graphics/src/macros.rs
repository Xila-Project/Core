#[macro_export]
macro_rules! lock {
    ($body:block) => {{
        let _lock = $crate::get_instance().lock().await;
        let __result = { $body };
        ::core::mem::drop(_lock);
        __result
    }};
}

#[macro_export]
macro_rules! synchronous_lock {
    ($body:block) => {{
        let _lock = $crate::get_instance().synchronous_lock();
        let __result = { $($body)* };
        ::core::mem::drop(_lock);
        __result
    }};
}
