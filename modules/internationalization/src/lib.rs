#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod range;

pub use range::*;

pub use internationalization_macros::translate;

const DEFAULT_LOCALE: &str = "en";
const DEFAULT_FALLBACK_LOCALE: &str = "en";

pub const fn get_locale() -> &'static str {
    match option_env!("INTERNATIONALIZATION_LOCALE") {
        Some(locale) => locale,
        None => DEFAULT_LOCALE,
    }
}

pub const fn get_fallback_locale() -> &'static str {
    match option_env!("INTERNATIONALIZATION_FALLBACK") {
        Some(locale) => locale,
        None => DEFAULT_FALLBACK_LOCALE,
    }
}

#[cfg(feature = "std")]
pub fn get_locale_build() -> std::string::String {
    use std::string::ToString;

    match std::env::var("INTERNATIONALIZATION_LOCALE") {
        Ok(locale) => locale,
        Err(_) => DEFAULT_LOCALE.to_string(),
    }
}

#[cfg(feature = "std")]
pub fn get_fallback_locale_build() -> std::string::String {
    use std::string::ToString;

    match std::env::var("INTERNATIONALIZATION_FALLBACK") {
        Ok(locale) => locale,
        Err(_) => DEFAULT_FALLBACK_LOCALE.to_string(),
    }
}
