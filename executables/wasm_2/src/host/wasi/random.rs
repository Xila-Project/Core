use wasmi::Caller;

use crate::{
    define_wasi_module,
    host::{store::GlobalStore, wasi::memory::get_memory},
};

define_wasi_module! {
    module: "wasi_snapshot_preview1";

    fn random_get(
        caller: Caller<GlobalStore>,
        buf_ptr: i32,
        buf_len: i32,
    ) -> Result<i32, wasmi::Error> {
        let old_state = caller.data().wasi.random_state;
        let mut buf = alloc::vec![0u8; buf_len as usize];
        let mut state = old_state;
        for byte in &mut buf {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *byte = (state >> 32) as u8;
        }

        let mut caller = caller;
        let memory = get_memory(&caller)?;
        {
            let data = memory.data_mut(&mut caller);
            data[buf_ptr as usize..buf_ptr as usize + buf_len as usize].copy_from_slice(&buf);
        }
        caller.data_mut().wasi.random_state = state;
        Ok(0)
    }
}
