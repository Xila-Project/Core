use wasmi::Caller;

use crate::{define_wasi_module, host::store::GlobalStore};

define_wasi_module! {
    module: "wasi_snapshot_preview1";

    fn sched_yield(
        _caller: Caller<GlobalStore>,
    ) -> Result<i32, wasmi::Error> {
        xila::task::block_on(xila::task::yield_now());
        Ok(0)
    }
}
