#![no_std]

mod error;

extern crate alloc;

use core::mem::forget;
use core::num::NonZeroUsize;

use crate::Error;
use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};
pub use error::*;
use xila::executable::{Standard, implement_executable_device};
use xila::file_system::{Mode, Path};
use xila::task;
use xila::virtual_file_system::{self, File};
use xila::virtual_machine;

pub struct WasmDevice;

implement_executable_device!(
    structure: WasmDevice,
    mount_path: "/binaries/wasm",
    main_function: main,
);

pub async fn inner_main(standard: &Standard, arguments: Vec<String>) -> Result<(), Error> {
    if arguments.len() != 1 {
        return Err(Error::InvalidNumberOfArguments);
    }

    let path = Path::new(&arguments[0]);

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

pub async fn main(standard: Standard, arguments: Vec<String>) -> Result<(), NonZeroUsize> {
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
