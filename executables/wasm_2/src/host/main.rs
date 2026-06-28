use core::{
    num::{NonZeroU32, NonZeroUsize},
    time::Duration,
};

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use core::fmt::Write;
use getargs_derive::GetArgs;
use wasmi::{Caller, Config, Engine, Linker, Module, Store, TypedResumableCall};
use xila::{
    executable::Standard,
    file_system::{Kind, Path},
    log, task,
    virtual_file_system::{self, File},
};

use crate::host::{
    error::{Error, Result},
    store::GlobalStore,
};

const DEFAULT_STACK_SIZE: usize = 4096;
const START_FUNCTION_NAME: &str = "_start";
const INSTALL_FUNCTION_NAME: &str = "__install";

#[derive(GetArgs)]
struct WasmArguments<'a> {
    path: &'a str,
    #[arg(flag)]
    install: bool,
    #[arg(default = DEFAULT_STACK_SIZE)]
    stack_size: usize,
    #[arg(default = NonZeroU32::new(200).unwrap())]
    instruction_limit: NonZeroU32,
}

pub async fn main(
    mut standard: Standard,
    arguments: Vec<String>,
) -> core::result::Result<(), NonZeroUsize> {
    let arguments = match WasmArguments::from_args(arguments.iter().map(|s| s.as_str())) {
        Ok(args) => args,
        Err(e) => {
            writeln!(standard.error(), "Error parsing arguments: {}", e).unwrap();
            return Err(NonZeroUsize::new(1).unwrap());
        }
    };

    main_inner(standard.duplicate().await.unwrap(), arguments)
        .await
        .map_err(|e| {
            writeln!(standard.error(), "Error: {:?}", e).unwrap();
            NonZeroUsize::new(1).unwrap()
        })?;

    Ok(())
}

async fn read_file(path: &str) -> Result<Vec<u8>> {
    let virtual_file_system = virtual_file_system::get_instance();

    let task = task::get_instance().get_current_task_identifier().await;

    let path = Path::new(path);

    let statistics = virtual_file_system.get_statistics(&path).await?;

    if statistics.kind != Kind::File {
        return Err(Error::NotAWasmFile);
    }

    let mut buffer = Vec::with_capacity(statistics.size as usize);

    File::read_from_path(virtual_file_system, task, &path, &mut buffer).await?;

    Ok(buffer)
}

pub async fn main_inner(standard: Standard, arguments: WasmArguments<'_>) -> Result<()> {
    let WasmArguments {
        path,
        install,
        stack_size,
        instruction_limit,
    } = arguments;

    let mut configuration = Config::default();
    configuration.consume_fuel(true);

    // First step is to create the Wasm execution engine with some config.
    //
    // In this example we are using the default configuration.
    let engine = Engine::new(&configuration);

    let buffer = read_file(path).await?;

    // Now we can compile the above Wasm module with the given Wasm source.
    let module = Module::new(&engine, &buffer)?;

    // Wasm objects operate within the context of a Wasm `Store`.
    //
    // Each `Store` has a type parameter to store host specific data.
    // In this example the host state is a simple `u32` type with value `42`.
    let mut store = Store::new(&engine, 42);

    // A linker can be used to instantiate Wasm modules.
    // The job of a linker is to satisfy the Wasm module's imports.
    let mut linker = Linker::<GlobalStore>::new(&engine);

    // We are required to define all imports before instantiating a Wasm module.
    linker.func_wrap(
        "host",
        "hello",
        |caller: Caller<'_, GlobalStore>, param: i32| {
            log::information!(
                "Hello from WebAssembly! Got {} from WebAssembly and my host state is: {}",
                param,
                caller.data()
            );
        },
    )?;

    let instance = linker.instantiate_and_start(&mut store, &module)?;
    // Now we can finally query the exported "hello" function and call it.

    let function_name = if arguments.install {
        INSTALL_FUNCTION_NAME
    } else {
        START_FUNCTION_NAME
    };

    let function = instance.get_typed_func::<(), i32>(&store, function_name)?;

    loop {
        store.set_fuel(arguments.instruction_limit.get() as u64)?;

        match function.call_resumable(&mut store, ()) {
            Ok(TypedResumableCall::Finished(r)) => {
                log::information!("Function finished with result: {:?}", r);
                if r == 0 {
                    break Ok(());
                } else {
                    break Err(Error::Runtime(r));
                }
            }
            Ok(TypedResumableCall::HostTrap(r)) => {
                log::information!("Function is resumable with result: {:?}", r);
                task::sleep(Duration::from_millis(10)).await;
            }
            Ok(TypedResumableCall::OutOfFuel(r)) => {
                log::information!("Function trapped with result: {:?}", r);
                task::sleep(Duration::from_millis(10)).await;
            }
            Err(e) => {
                break Err(Error::Wasm(e));
            }
        }
    }
}
