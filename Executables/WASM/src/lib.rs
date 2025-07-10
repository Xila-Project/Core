#![no_std]

mod error;

extern crate alloc;

use core::mem::forget;
use core::num::NonZeroUsize;

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};
use executable::Standard;
use file_system::{Mode, Path};

use virtual_file_system::File;

use crate::Error;

pub use error::*;
use executable::implement_executable_device;

pub struct WasmDeviceType;

implement_executable_device!(
    Structure: WasmDeviceType,
    Mount_path: "/Binaries/WASM",
    Main_function: main,
);

pub async fn inner_main(standard: &Standard, arguments: String) -> Result<(), Error> {
    let arguments = arguments.split_whitespace().collect::<Vec<&str>>();

    if arguments.len() != 1 {
        return Err(Error::InvalidNumberOfArguments);
    }

    let path = Path::new(arguments[0]);

    match path.get_extension() {
        Some("wasm") | Some("WASM") => Ok(()),
        _ => return Err(Error::NotAWasmFile),
    }?;

    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        let current_path = task::get_instance()
            .get_environment_variable(standard.get_task(), "Current_directory")
            .await
            .map_err(|_| Error::FailedToGetCurrentDirectory)?;

        let current_path = current_path.get_value();

        let current_path = Path::new(current_path);

        current_path.join(path).ok_or(Error::InvalidPath)?
    };

    let file = File::open(
        virtual_file_system::get_instance(),
        &path,
        Mode::READ_ONLY.into(),
    )
    .await
    .map_err(|_| Error::FailedToOpenFile)?;

    let size: usize = file
        .get_statistics()
        .await
        .map_err(|_| Error::FailedToOpenFile)?
        .get_size()
        .into();

    let mut buffer = Vec::with_capacity(size);

    file.read_to_end(&mut buffer)
        .await
        .map_err(|_| Error::FailedToReadFile)?;

    let (standard_in, standard_out, standard_error) = standard.split();

    virtual_machine::get_instance()
        .execute(buffer, 4096, standard_in, standard_out, standard_error)
        .await
        .map_err(|_| Error::FailedToExecute)?;

    Ok(())
}

pub async fn main(standard: Standard, arguments: String) -> Result<(), NonZeroUsize> {
    match inner_main(&standard, arguments).await {
        Ok(()) => {
            forget(standard);
            Ok(())
        }
        Err(error) => {
            standard.print_error_line(&error.to_string()).await;
            Err(error.into())
        }
    }
}
