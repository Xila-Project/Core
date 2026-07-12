use wasmi::Caller;

use crate::{
    define_wasi_module,
    host::{store::GlobalStore, wasi::memory::get_memory},
};

define_wasi_module! {
    module: "wasi_snapshot_preview1";

    fn poll_oneoff(
        caller: Caller<GlobalStore>,
        _in_ptr: i32,
        _out_ptr: i32,
        nsubscriptions: i32,
        nevents_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        let mut caller = caller;
        let memory = get_memory(&caller)?;

        for _ in 0..nsubscriptions {
            xila::task::block_on(xila::task::yield_now());
        }

        let data = memory.data_mut(&mut caller);
        data[nevents_ptr as usize..nevents_ptr as usize + 4].copy_from_slice(&0i32.to_le_bytes());
        Ok(0)
    }
}
