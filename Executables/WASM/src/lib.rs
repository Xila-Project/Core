#![no_std]
#![allow(non_camel_case_types)]

mod error;

extern crate alloc;

use core::mem::forget;
use core::num::NonZeroUsize;

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};
use executable::Standard_type;
use file_system::{Mode_type, Path_type};

use virtual_file_system::File_type;

use crate::Error_type;

pub use error::*;
use executable::Implement_executable_device;

pub struct WASM_device_type;

Implement_executable_device!(
    Structure: WASM_device_type,
    Mount_path: "/Binaries/WASM",
    Main_function: main,
);

pub async fn inner_main(standard: &Standard_type, arguments: String) -> Result<(), Error_type> {
    let arguments = arguments.split_whitespace().collect::<Vec<&str>>();

    if arguments.len() != 1 {
        return Err(Error_type::Invalid_number_of_arguments);
    }

    let path = Path_type::new(arguments[0]);

    match path.get_extension() {
        Some("wasm") | Some("WASM") => Ok(()),
        _ => return Err(Error_type::Not_a_WASM_file),
    }?;

    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        let current_path = task::get_instance()
            .get_environment_variable(standard.get_task(), "Current_directory")
            .await
            .map_err(|_| Error_type::Failed_to_get_current_directory)?;

        let current_path = current_path.get_value();

        let current_path = Path_type::new(current_path);

        current_path.join(path).ok_or(Error_type::Invalid_path)?
    };

    let file = File_type::open(
        virtual_file_system::get_instance(),
        &path,
        Mode_type::READ_ONLY.into(),
    )
    .await
    .map_err(|_| Error_type::Failed_to_open_file)?;

    let size: usize = file
        .get_statistics()
        .await
        .map_err(|_| Error_type::Failed_to_open_file)?
        .get_size()
        .into();

    let mut buffer = Vec::with_capacity(size);

    file.read_to_end(&mut buffer)
        .await
        .map_err(|_| Error_type::Failed_to_read_file)?;

    let (standard_in, standard_out, standard_error) = standard.split();

    virtual_machine::get_instance()
        .execute(buffer, 4096, standard_in, standard_out, standard_error)
        .await
        .map_err(|_| Error_type::Failed_to_execute)?;

    Ok(())
}

pub async fn main(standard: Standard_type, arguments: String) -> Result<(), NonZeroUsize> {
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
