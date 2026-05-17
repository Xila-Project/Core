macro_rules! abi_function {
    (
        $(#[$meta:meta])*
        fn $name:ident ( $($arg:ident : $ty:ty),* $(,)? ) -> XilaFileSystemResult $body:block
    ) => {
        $(#[$meta])*
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name ( $($arg : $ty),* ) -> XilaFileSystemResult {
            into_result(|| unsafe {
                $body
            })
        }
    };
}
