use core::fmt;

use Log::Logger_trait;

pub struct Logger_type;

impl Logger_trait for Logger_type {
    fn Enabled(&self, Level: Log::Level_type) -> bool {
        match Level {
            Log::Level_type::Error => true,
            Log::Level_type::Warn => true,
            Log::Level_type::Info => true,
            Log::Level_type::Debug => false,
            Log::Level_type::Trace => false,
        }
    }

    fn Write(&self, args: fmt::Arguments) {
        println!("{}", args);
    }
}

#[cfg(test)]
mod Tests {
    use super::*;
    use Log;

    static Logger: Logger_type = Logger_type;

    #[test]
    fn Test_write() {
        Log::Test_write(&Logger);
    }

    #[test]
    fn Test_log() {
        Log::Test_log(&Logger);
    }

    #[test]
    fn Test_flush() {
        Log::Test_flush(&Logger);
    }
}
