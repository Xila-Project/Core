#![no_std]
#![cfg(not(target_arch = "wasm32"))]

mod error;

extern crate alloc;

use crate::Error;
use alloc::boxed::Box;
use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::num::NonZeroUsize;
use core::{mem::forget, pin::Pin};
use xila::executable::ArgumentsParser;
use xila::executable::{Standard, implement_executable_device};
use xila::file_system::{Mode, Path};
use xila::synchronization::once_lock::OnceLock;
use xila::task::{self, SpawnerIdentifier};
use xila::virtual_file_system::{self, File};
use xila::virtual_machine;

pub use error::*;

pub struct WasmDevice;

type NewThreadExecutor =
    fn() -> Pin<Box<dyn core::future::Future<Output = SpawnerIdentifier> + Send>>;

static NEW_THREAD_EXECUTOR: OnceLock<NewThreadExecutor> = OnceLock::new();

const DEFAULT_STACK_SIZE: usize = 4096;

impl WasmDevice {
    pub fn new(new_thread_executor: Option<NewThreadExecutor>) -> Self {
        if let Some(new_thread_executor) = new_thread_executor {
            let _ = NEW_THREAD_EXECUTOR.init(new_thread_executor);
        }

        Self
    }
}

implement_executable_device!(
    structure: WasmDevice,
    mount_path: "/binaries/wasm",
    main_function: main,
);

pub async fn inner_main(standard: &Standard, arguments: Vec<String>) -> Result<(), Error> {
    let parsed_arguments = ArgumentsParser::new(&arguments);

    let install = parsed_arguments
        .clone()
        .any(|a| a.options.get_option("install").is_some());
    let stack_size = parsed_arguments
        .clone()
        .find_map(|a| {
            a.options
                .get_option("stack-size")
                .and_then(|s| s.parse::<usize>().ok())
        })
        .unwrap_or(DEFAULT_STACK_SIZE);
    let path = parsed_arguments
        .last()
        .and_then(|arg| arg.value.map(Path::new))
        .ok_or(Error::MissingArgument("path"))?;

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

    let function_name = if install { Some("__install") } else { None };

    if let Some(new_thread_executor) = NEW_THREAD_EXECUTOR.try_get() {
        let spawner_identifier = new_thread_executor().await;

        let standard = standard
            .duplicate()
            .await
            .map_err(Error::FailedToDuplicateStandard)?;

        task::get_instance()
            .spawn(
                standard.get_task(),
                "WASM Execution",
                Some(spawner_identifier),
                move |task_identifier| async move {
                    //log::information!("WASM task identifier: {task_identifier:?}");

                    let standard = standard
                        .transfer(task_identifier)
                        .await
                        .map_err(Error::FailedToTransferStandard)?;

                    let (standard_in, standard_out, standard_error) = standard.split();

                    virtual_machine::get_instance()
                        .execute(
                            buffer,
                            stack_size,
                            (standard_in, standard_out, standard_error),
                            function_name,
                            vec![],
                        )
                        .await
                        .map_err(Error::FailedToExecute)
                },
            )
            .await
            .map_err(Error::FailedToSpawnTask)?;
    } else {
        let (standard_in, standard_out, standard_error) = standard.split();

        virtual_machine::get_instance()
            .execute(
                buffer,
                stack_size,
                (standard_in, standard_out, standard_error),
                function_name,
                vec![],
            )
            .await
            .map_err(Error::FailedToExecute)?;
    }

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
