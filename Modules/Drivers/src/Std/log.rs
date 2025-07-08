use core::fmt;

use log::Logger_trait;

pub struct Logger_type;

impl Logger_trait for Logger_type {
    fn enabled(&self, level: Log::Level_type) -> bool {
        match level {
            Log::Level_type::Error => true,
            Log::Level_type::Warn => true,
            Log::Level_type::Info => true,
            Log::Level_type::Debug => false,
            Log::Level_type::Trace => false,
        }
    }

    fn Write(&self, args: fmt::Arguments) {
        println!("{args}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Log;

    static Logger: Logger_type = Logger_type;

    #[test]
    fn test_write() {
        Log::Test_write(&Logger);
    }

    #[test]
    fn test_log() {
        Log::Test_log(&Logger);
    }

    #[test]
    fn test_flush() {
        Log::Test_flush(&Logger);
    }
}
