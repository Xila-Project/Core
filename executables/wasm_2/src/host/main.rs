use core::num::{NonZeroU32, NonZeroUsize};

use alloc::{string::String, vec::Vec};
use core::fmt::Write;
use getargs_derive::GetArgs;
use wasmi::{Caller, Engine, Linker, Module, Store};
use xila::{executable::Standard, log};

use crate::host::error::Result;

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

pub async fn main_inner(standard: Standard, arguments: WasmArguments<'_>) -> Result<()> {
    let wasm = r#"
        (module
            (import "host" "hello" (func $host_hello (param i32)))
            (func (export "hello")
                (call $host_hello (i32.const 3))
            )
        )
    "#;
    // First step is to create the Wasm execution engine with some config.
    //
    // In this example we are using the default configuration.
    let engine = Engine::default();
    // Now we can compile the above Wasm module with the given Wasm source.
    let module = Module::new(&engine, wasm)?;

    // Wasm objects operate within the context of a Wasm `Store`.
    //
    // Each `Store` has a type parameter to store host specific data.
    // In this example the host state is a simple `u32` type with value `42`.
    type HostState = u32;
    let mut store = Store::new(&engine, 42);

    // A linker can be used to instantiate Wasm modules.
    // The job of a linker is to satisfy the Wasm module's imports.
    let mut linker = <Linker<HostState>>::new(&engine);
    // We are required to define all imports before instantiating a Wasm module.
    linker.func_wrap(
        "host",
        "hello",
        |caller: Caller<'_, HostState>, param: i32| {
            log::information!(
                "Hello from WebAssembly! Got {} from WebAssembly and my host state is: {}",
                param,
                caller.data()
            );
        },
    )?;
    let instance = linker.instantiate_and_start(&mut store, &module)?;
    // Now we can finally query the exported "hello" function and call it.
    instance
        .get_typed_func::<(), ()>(&store, "hello")?
        .call(&mut store, ())?;
    Ok(())
}
