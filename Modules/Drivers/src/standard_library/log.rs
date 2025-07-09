use core::fmt;

use log::Logger_trait;

pub struct Logger_type;

impl Logger_trait for Logger_type {
    fn enabled(&self, level: log::Level_type) -> bool {
        match level {
            log::Level_type::Error => true,
            log::Level_type::Warn => true,
            log::Level_type::Info => true,
            log::Level_type::Debug => false,
            log::Level_type::Trace => false,
        }
    }

    fn write(&self, args: fmt::Arguments) {
        println!("{args}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log;

    static LOGGER: Logger_type = Logger_type;

    #[test]
    fn test_write() {
        log::test_write(&LOGGER);
    }

    #[test]
    fn test_log() {
        log::test_log(&LOGGER);
    }

    #[test]
    fn test_flush() {
        log::test_flush(&LOGGER);
    }
}
