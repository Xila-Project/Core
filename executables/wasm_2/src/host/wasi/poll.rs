use wasmi::Caller;

use crate::{define_wasi_module, host::store::GlobalStore};

define_wasi_module! {
    module: "wasi_snapshot_preview1";

    fn poll_oneoff(
        caller: Caller<GlobalStore>,
        _in_ptr: i32,
        _out_ptr: i32,
        nsubscriptions: i32,
        nevents_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        todo!("Implement poll_oneoff");
    }
}
