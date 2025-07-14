#![no_std]

extern crate alloc;

use alloc::fmt;
use log_external::Log;
use log_external::Metadata;
pub use log_external::Record;
pub use log_external::debug as Debug;
pub use log_external::error as Error;
pub use log_external::info as Information;
use log_external::set_logger;
use log_external::set_max_level;
pub use log_external::trace as Trace;
pub use log_external::warn as Warning;
use synchronization::once_lock::OnceLock;

const BOLD: &str = "\x1b[0;1m";
const RED: &str = "\x1b[0;41m";
const YELLOW: &str = "\x1b[0;43m";
const BLUE: &str = "\x1b[0;46m";
const GREEN: &str = "\x1b[0;42m";
const GREY: &str = "\x1b[0;100m";
const RESET: &str = "\x1b[0m";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<log_external::Level> for Level {
    fn from(level: log_external::Level) -> Self {
        match level {
            log_external::Level::Error => Level::Error,
            log_external::Level::Warn => Level::Warn,
            log_external::Level::Info => Level::Info,
            log_external::Level::Debug => Level::Debug,
            log_external::Level::Trace => Level::Trace,
        }
    }
}

pub trait LoggerTrait: Send + Sync {
    fn enabled(&self, level: Level) -> bool;

    fn write(&self, arguments: fmt::Arguments);

    fn log(&self, record: &Record) {
        let letter = match record.level() {
            log_external::Level::Error => "E",
            log_external::Level::Warn => "W",
            log_external::Level::Info => "I",
            log_external::Level::Debug => "D",
            log_external::Level::Trace => "T",
        };

        let color = match record.level() {
            log_external::Level::Error => RED,
            log_external::Level::Warn => YELLOW,
            log_external::Level::Info => BLUE,
            log_external::Level::Debug => GREEN,
            log_external::Level::Trace => GREY,
        };

        self.write(format_args!(
            "{} {} {} | {}{}{} | {}",
            color,
            letter,
            RESET,
            BOLD,
            record.target(),
            RESET,
            record.args()
        ))
    }

    fn flush(&self) {
        // Implement flush logic if needed, e.g., flushing buffers to a file or console
    }
}

struct LoggerType(&'static dyn LoggerTrait);

impl Log for LoggerType {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.0.enabled(Level::from(metadata.level()))
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        self.0.log(record)
    }

    fn flush(&self) {
        self.0.flush();
    }
}

static LOGGER_INSTANCE: OnceLock<LoggerType> = OnceLock::new();

pub fn initialize(logger: &'static dyn LoggerTrait) -> Result<(), log_external::SetLoggerError> {
    let logger = LOGGER_INSTANCE.get_or_init(|| LoggerType(logger));

    set_logger(logger).expect("Failed to set logger");
    set_max_level(log_external::LevelFilter::Trace);
    Ok(())
}

pub fn test_write(logger: &impl LoggerTrait) {
    for i in 0..5 {
        logger.write(format_args!("This is a test message number {i}."));
    }
}

pub fn test_log(logger: &impl LoggerTrait) {
    logger.log(
        &Record::builder()
            .level(log_external::Level::Info)
            .args(format_args!("This is a test log message."))
            .build(),
    );
    logger.log(
        &Record::builder()
            .level(log_external::Level::Warn)
            .args(format_args!("This is a test warning message."))
            .build(),
    );
    logger.log(
        &Record::builder()
            .level(log_external::Level::Error)
            .args(format_args!("This is a test error message."))
            .build(),
    );
    logger.log(
        &Record::builder()
            .level(log_external::Level::Debug)
            .args(format_args!("This is a test debug message."))
            .build(),
    );
    logger.log(
        &Record::builder()
            .level(log_external::Level::Trace)
            .args(format_args!("This is a test trace message."))
            .build(),
    );
}

pub fn test_flush(logger: &impl LoggerTrait) {
    logger.flush();
}
