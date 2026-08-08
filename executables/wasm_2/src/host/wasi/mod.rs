mod context;
mod directory;
mod environment;
mod error;
mod file;
mod path;
mod poll;
mod process;
mod random;
pub mod register;
mod scheduling;
mod socket;
mod time;
mod types;

pub use context::*;
pub use error::Error;

#[macro_export]
macro_rules! wasi_result {
    ( $($tokens:tt)* ) => {
        // Wrapping the expansion in an outer block turns it into an expression
        {
            let __result: core::result::Result<(), $crate::host::wasi::Error> = {
                $($tokens)*
            };

            match __result {
                Ok(()) => Ok(0),
                Err(err) => Ok(core::num::NonZeroI32::from(err).into()),
            }
        }
    };
}
