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

pub async fn Inner_main(Standard: &Standard_type, Arguments: String) -> Result<(), Error_type> {
    let Arguments = Arguments.split_whitespace().collect::<Vec<&str>>();

    if Arguments.len() != 1 {
        return Err(Error_type::Invalid_number_of_arguments);
    }

    let Path = Path_type::New(Arguments[0]);

    match Path.Get_extension() {
        Some("wasm") | Some("WASM") => Ok(()),
        _ => return Err(Error_type::Not_a_WASM_file),
    }?;

    let Path = if Path.Is_absolute() {
        Path.to_owned()
    } else {
        let Current_path = Task::Get_instance()
            .Get_environment_variable(Standard.Get_task(), "Current_directory")
            .await
            .map_err(|_| Error_type::Failed_to_get_current_directory)?;

        let Current_path = Current_path.Get_value();

        let Current_path = Path_type::New(Current_path);

        Current_path.Join(Path).ok_or(Error_type::Invalid_path)?
    };

    let File = File_type::Open(
        Virtual_file_system::Get_instance(),
        &Path,
        Mode_type::Read_only.into(),
    )
    .await
    .map_err(|_| Error_type::Failed_to_open_file)?;

    let Size: usize = File
        .Get_statistics()
        .await
        .map_err(|_| Error_type::Failed_to_open_file)?
        .Get_size()
        .into();

    let mut Buffer = Vec::with_capacity(Size);

    File.Read_to_end(&mut Buffer)
        .await
        .map_err(|_| Error_type::Failed_to_read_file)?;

    let (Standard_in, Standard_out, Standard_error) = Standard.Split();

    Virtual_machine::Get_instance()
        .Execute(Buffer, 4096, Standard_in, Standard_out, Standard_error)
        .await
        .map_err(|_| Error_type::Failed_to_execute)?;

    Ok(())
}

pub async fn Main(Standard: Standard_type, Arguments: String) -> Result<(), NonZeroUsize> {
    match Inner_main(&Standard, Arguments).await {
        Ok(()) => {
            forget(Standard);
            Ok(())
        }
        Err(Error) => {
            Standard.Print_error_line(&Error.to_string()).await;
            Err(Error.into())
        }
    }
}
