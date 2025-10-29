#[cfg(feature = "std")]
mod generator;

#[cfg(feature = "std")]
pub use generator::{Configuration, generate_translations};

#[macro_export]
macro_rules! include_translations {
    () => {
        mod translations {
            include!(concat!(env!("OUT_DIR"), "/internationalization.rs"));
        }
    };
    ($module_name:ident) => {
        mod $module_name {
            include!(concat!(env!("OUT_DIR"), "/internationalization.rs"));
        }
    };
}
