use alloc::format;
use log::{Level, LoggerTrait, Record};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn warn(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn info(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn debug(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn trace(s: &str);
}

pub struct Logger;

impl LoggerTrait for Logger {
    fn enabled(&self, _: Level) -> bool {
        true
    }

    fn write(&self, _: core::fmt::Arguments) {}

    fn log(&self, record: &Record) {
        let message = format!("{} | {}", record.target, record.arguments);
        match record.level {
            Level::Error => error(&message),
            Level::Warn => warn(&message),
            Level::Info => info(&message),
            Level::Debug => (),
            Level::Trace => (),
        }
    }
}
