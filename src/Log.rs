use std::time::Instant;

#[cfg(any(target_os = "linux", target_os = "espidf"))]
include!("Log/Standard_library.rs");

#[macro_export]
macro_rules! Write_format {
    ($log_type:expr, $color:expr, $($arg:tt)*) => {
        println!("{}{} : {}{}", $color, $log_type, format_args!($($arg)*), "\x1b[0m");
    };
}

#[macro_export]
macro_rules! Error {
    ($($arg:tt)*) => {
        Write_format!("Error", "\x1b[31m", $($arg)*);
    };
}

#[macro_export]
macro_rules! Warning {
    ($($arg:tt)*) => {
        Write_format!("Warning", "\x1b[33m", $($arg)*);
    };
}

#[macro_export]
macro_rules! Information {
    ($($arg:tt)*) => {
        Write_format!("Information", "\x1b[34m", $($arg)*);
    };
}

#[macro_export]
macro_rules! Debug {
    ($($arg:tt)*) => {
        Write_format!("Debug", "\x1b[32m", $($arg)*);
    };
}
