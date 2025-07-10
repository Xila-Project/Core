use core::fmt;

use log::LoggerTrait;

pub struct LoggerType;

impl LoggerTrait for LoggerType {
    fn enabled(&self, level: log::Level) -> bool {
        match level {
            log::Level::Error => true,
            log::Level::Warn => true,
            log::Level::Info => true,
            log::Level::Debug => false,
            log::Level::Trace => false,
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

    static LOGGER: LoggerType = LoggerType;

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
