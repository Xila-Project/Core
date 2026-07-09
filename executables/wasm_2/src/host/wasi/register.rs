use wasmi::Linker;

use crate::host::{error::Result, store::GlobalStore};

const MODULE_NAME: &str = "wasi_snapshot_preview1";

pub fn add_wasi_to_linker(linker: &mut Linker<GlobalStore>) -> Result<()> {
    linker.func_wrap(
        MODULE_NAME,
        "args_get",
        crate::host::wasi::environment::args_get,
    )?;

    Ok(())
}
