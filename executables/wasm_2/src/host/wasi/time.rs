use wasmi::Caller;

use crate::{
    define_wasi_module,
    host::{store::GlobalStore, wasi::memory::get_memory},
};

define_wasi_module! {
    module: "wasi_snapshot_preview1";

    fn clock_res_get(
        caller: Caller<GlobalStore>,
        _clock_id: i32,
        resolution_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        let mut caller = caller;
        let memory = get_memory(&caller)?;
        let resolution = xila::time::get_instance()
            .get_current_time_since_startup()
            .map_err(|_| wasmi::Error::new("time error"))?;
        let data = memory.data_mut(&mut caller);
        data[resolution_ptr as usize..resolution_ptr as usize + 8]
            .copy_from_slice(&(resolution.as_nanos() as u64).to_le_bytes());
        Ok(0)
    }

    fn clock_time_get(
        caller: Caller<GlobalStore>,
        _clock_id: i32,
        _precision: i64,
        time_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        let mut caller = caller;
        let memory = get_memory(&caller)?;
        let now = xila::time::get_instance()
            .get_current_time()
            .map_err(|_| wasmi::Error::new("time error"))?;
        let data = memory.data_mut(&mut caller);
        data[time_ptr as usize..time_ptr as usize + 8]
            .copy_from_slice(&(now.as_nanos() as u64).to_le_bytes());
        Ok(0)
    }
}
