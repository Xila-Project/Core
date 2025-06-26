#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate alloc;

use Synchronization::once_lock::OnceLock;
use alloc::fmt;
use log::Log;
use log::Metadata;
pub use log::Record;
pub use log::debug as Debug;
pub use log::error as Error;
pub use log::info as Information;
use log::set_logger;
use log::set_max_level;
pub use log::trace as Trace;
pub use log::warn as Warning;

const Error_icon: &str = "❌";
const Warning_icon: &str = "⚠️";
const Information_icon: &str = "ℹ️";
const Debug_icon: &str = "🐞";
const Trace_icon: &str = "🐾";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level_type {
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<log::Level> for Level_type {
    fn from(Level: log::Level) -> Self {
        match Level {
            log::Level::Error => Level_type::Error,
            log::Level::Warn => Level_type::Warn,
            log::Level::Info => Level_type::Info,
            log::Level::Debug => Level_type::Debug,
            log::Level::Trace => Level_type::Trace,
        }
    }
}

pub trait Logger_trait: Send + Sync {
    fn Enabled(&self, Level: Level_type) -> bool;

    fn Write(&self, Arguments: fmt::Arguments);

    fn Log(&self, Record: &Record) {
        self.Write(format_args!(
            "{} {}: {}",
            match Record.level() {
                log::Level::Error => Error_icon,
                log::Level::Warn => Warning_icon,
                log::Level::Info => Information_icon,
                log::Level::Debug => Debug_icon,
                log::Level::Trace => Trace_icon,
            },
            Record.target(),
            Record.args()
        ))
    }

    fn Flush(&self) {
        // Implement flush logic if needed, e.g., flushing buffers to a file or console
    }
}

struct Logger_type(&'static dyn Logger_trait);

impl Log for Logger_type {
    fn enabled(&self, Metadata: &Metadata) -> bool {
        self.0.Enabled(Level_type::from(Metadata.level()))
    }

    fn log(&self, Record: &Record) {
        if !self.enabled(Record.metadata()) {
            return;
        }
        self.0.Log(Record)
    }

    fn flush(&self) {
        self.0.Flush();
    }
}

static Logger_instance: OnceLock<Logger_type> = OnceLock::new();

pub fn Initialize(Logger: &'static dyn Logger_trait) -> Result<(), log::SetLoggerError> {
    let Logger = Logger_instance.get_or_init(|| Logger_type(Logger));

    set_logger(Logger).expect("Failed to set logger");
    set_max_level(log::LevelFilter::Trace);
    Ok(())
}

pub fn Test_write(Logger: &impl Logger_trait) {
    for i in 0..5 {
        Logger.Write(format_args!("This is a test message number {}.", i));
    }
}

pub fn Test_log(Logger: &impl Logger_trait) {
    Logger.Log(
        &Record::builder()
            .level(log::Level::Info)
            .args(format_args!("This is a test log message."))
            .build(),
    );
    Logger.Log(
        &Record::builder()
            .level(log::Level::Warn)
            .args(format_args!("This is a test warning message."))
            .build(),
    );
    Logger.Log(
        &Record::builder()
            .level(log::Level::Error)
            .args(format_args!("This is a test error message."))
            .build(),
    );
    Logger.Log(
        &Record::builder()
            .level(log::Level::Debug)
            .args(format_args!("This is a test debug message."))
            .build(),
    );
    Logger.Log(
        &Record::builder()
            .level(log::Level::Trace)
            .args(format_args!("This is a test trace message."))
            .build(),
    );
}

pub fn Test_flush(Logger: &impl Logger_trait) {
    Logger.Flush();
}
