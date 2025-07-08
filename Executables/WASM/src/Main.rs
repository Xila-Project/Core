use core::mem::forget;
use core::num::NonZeroUsize;

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};
use Executable::Standard_type;
use File_system::{Mode_type, Path_type};

use Virtual_file_system::File_type;

use crate::Error_type;

pub async fn inner_main(standard: &Standard_type, arguments: String) -> Result<(), Error_type> {
    let arguments = arguments.split_whitespace().collect::<Vec<&str>>();

    if arguments.len() != 1 {
        return Err(Error_type::Invalid_number_of_arguments);
    }

    let path = Path_type::New(arguments[0]);

    match path.get_extension() {
        Some("wasm") | Some("WASM") => Ok(()),
        _ => return Err(Error_type::Not_a_WASM_file),
    }?;

    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        let current_path = Task::get_instance()
            .get_environment_variable(standard.get_task(), "Current_directory")
            .await
            .map_err(|_| Error_type::Failed_to_get_current_directory)?;

        let current_path = current_path.get_value();

        let current_path = Path_type::New(current_path);

        current_path.Join(path).ok_or(Error_type::Invalid_path)?
    };

    let file = File_type::open(
        Virtual_file_system::get_instance(),
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

    Virtual_machine::get_instance()
        .Execute(buffer, 4096, standard_in, standard_out, standard_error)
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
