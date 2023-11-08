#[macro_export]
macro_rules! Write {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

