mod bindings;
mod virtual_machine;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};
use core::fmt::Write;
use core::num::NonZeroUsize;
use core::pin::Pin;
use xila::executable::ArgumentsParser;
use xila::executable::ExecutableTrait;
use xila::executable::MainFuture;
use xila::executable::Standard;
use xila::file_system::{Kind, Path};
use xila::synchronization::once_lock::OnceLock;
use xila::task::{self, SpawnerIdentifier};
use xila::virtual_file_system::{self, File};

#[cfg(feature = "graphics")]
use crate::host::bindings::graphics::GraphicsBindings;
use crate::host::virtual_machine::{Error, Registrable};

pub struct WasmExecutable;

type NewThreadExecutor =
    fn() -> Pin<Box<dyn core::future::Future<Output = SpawnerIdentifier> + Send>>;

static NEW_THREAD_EXECUTOR: OnceLock<NewThreadExecutor> = OnceLock::new();
static RUNTIME: OnceLock<virtual_machine::Runtime> = OnceLock::new();
const REGISTRABLES: &[&dyn Registrable] = &[
    #[cfg(feature = "graphics")]
    &GraphicsBindings,
];
const START_FUNCTION_NAME: &str = "_start";
const INSTALL_FUNCTION_NAME: &str = "__install";
const DEFAULT_STACK_SIZE: usize = 4096;

impl WasmExecutable {
    pub fn new(new_thread_executor: Option<NewThreadExecutor>) -> Self {
        if let Some(new_thread_executor) = new_thread_executor {
            let _ = NEW_THREAD_EXECUTOR.init(new_thread_executor);
        }

        Self
    }
}

impl ExecutableTrait for WasmExecutable {
    fn main(standard: Standard, arguments: Vec<String>) -> MainFuture {
        Box::pin(async move { main(standard, arguments).await })
    }
}

pub async fn inner_main(standard: Standard, arguments: Vec<String>) -> Result<(), Error> {
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

    let task = task::get_instance().get_current_task_identifier().await;

    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        let current_path = task::get_instance()
            .get_environment_variable(task, "Current_directory")
            .await
            .map_err(|_| Error::FailedToGetCurrentDirectory)?;

        let current_path = current_path.get_value();

        let current_path = Path::new(current_path);

        current_path.join(path).ok_or(Error::InvalidPath)?
    };

    let virtual_file_system = virtual_file_system::get_instance();

    let statistics = virtual_file_system
        .get_statistics(&path)
        .await
        .map_err(|_| Error::InvalidPath)?;

    if statistics.kind != Kind::File {
        return Err(Error::NotAWasmFile);
    }

    let mut buffer = Vec::with_capacity(statistics.size as usize);

    File::read_from_path(virtual_file_system, task, &path, &mut buffer)
        .await
        .map_err(|_| Error::FailedToReadFile)?;

    let name = path.get_file_name().to_string();

    let function_name = if install {
        INSTALL_FUNCTION_NAME
    } else {
        START_FUNCTION_NAME
    };

    let runtime = RUNTIME
        .get_or_init(|| virtual_machine::Runtime::new(REGISTRABLES.iter().copied()).unwrap());

    let standard = standard.split();

    if let Some(new_thread_executor) = NEW_THREAD_EXECUTOR.try_get() {
        let spawner_identifier = new_thread_executor().await;

        task::get_instance()
            .spawn(
                task,
                "WASM Execution",
                Some(spawner_identifier),
                move |task_identifier| async move {
                    runtime
                        .execute(
                            &name,
                            buffer,
                            stack_size,
                            standard,
                            function_name,
                            vec![],
                            task_identifier,
                        )
                        .await
                },
            )
            .await
            .map_err(Error::FailedToSpawnTask)?
            .0
            .join()
            .await?;
    } else {
        let task_identifier = task::get_instance().get_current_task_identifier().await;

        runtime
            .execute(
                &name,
                buffer,
                stack_size,
                standard,
                function_name,
                vec![],
                task_identifier,
            )
            .await?;
    }

    Ok(())
}

pub async fn main(standard: Standard, arguments: Vec<String>) -> Result<(), NonZeroUsize> {
    let mut duplicated_standard = standard
        .duplicate()
        .await
        .map_err(Error::FailedToDuplicateStandard)?;

    match inner_main(standard, arguments).await {
        Ok(()) => Ok(()),
        Err(error) => {
            writeln!(
                duplicated_standard.standard_error,
                "WASM Executable Error: {}",
                error
            )
            .unwrap();
            Err(error.into())
        }
    }
}
