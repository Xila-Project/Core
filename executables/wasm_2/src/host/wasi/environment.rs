use wasmi::{Caller, Memory};
use xila::task::{self, block_on};

use crate::{
    define_wasi_module,
    host::{
        store::GlobalStore,
        wasi::{context::WasiContext, error::WasiResult},
    },
    wasi_result,
};

fn get_memory(caller: Caller<GlobalStore>) -> Result<Memory, wasmi::Error> {
    caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| wasmi::Error::new("missing memory"))
}

define_wasi_module! {


 fn args_get(
    mut caller: Caller<GlobalStore>,
    argc_ptr: i32,
    argv_buf_size_ptr: i32,
) -> Result<i32, wasmi::Error> {
    let memory = get_memory(caller)?;

    let data = memory.data_mut(&mut caller);
    let arguments = caller.data().wasi.args.clone();

    wasi_result! {
        let mut argv_offset = argc_ptr as usize;
        let mut buf_offset = argv_buf_size_ptr as usize;

        for arg in &arguments {
            data[argv_offset..argv_offset + 4]
                .copy_from_slice(&(buf_offset as i32).to_le_bytes());
            argv_offset += 4;

            data[buf_offset..buf_offset + arg.len()].copy_from_slice(arg);
            buf_offset += arg.len();
            data[buf_offset] = 0;
            buf_offset += 1;
        }

        Ok(())
    }
}

 fn args_sizes_get(
    mut caller: Caller<GlobalStore>,
    argc_ptr: i32,
    argv_buf_size_ptr: i32,
) -> Result<i32, wasmi::Error> {
    let memory = get_memory(caller)?;

    let data = memory.data_mut(&mut caller);
    let arguments = caller.data().wasi.args.clone();

    wasi_result! {
        let argc = arguments.len() as i32;
        let argv_buf_size = arguments.iter().map(|a| a.len() + 1).sum::<usize>() as i32;

        data[argc_ptr as usize..argc_ptr as usize + 4].copy_from_slice(&argc.to_le_bytes());
        data[argv_buf_size_ptr as usize..argv_buf_size_ptr as usize + 4].copy_from_slice(&argv_buf_size.to_le_bytes());

        Ok(())
    }
}

fn environ_get(
    mut caller: Caller<GlobalStore>,
    environ: i32,
    environ_buf: i32,
) -> Result<i32, wasmi::Error> {
    let memory = get_memory(caller)?;

    let data = memory.data_mut(&mut caller);
    let task = caller.data().wasi.task;

    let environment_variables = block_on(task::get_instance().get_environment_variables(task))
        .map_err(|_| wasmi::Error::new("Failed to get environment variables"))?;

    wasi_result! {

        let mut environ_offset = environ as usize;
        let mut environ_buf_offset = environ_buf as usize;

        for variable in &environment_variables {
            data[environ_offset..environ_offset + 4]
                .copy_from_slice(&(environ_buf_offset as i32).to_le_bytes());
            environ_offset += 4;

            let name = variable.get_name().as_bytes();
            let name_length = variable.get_name().len();
            let value = variable.get_value().as_bytes() ;
            let value_length = variable.get_value().len();

            data[environ_buf_offset..environ_buf_offset + name_length].copy_from_slice(
               name
            );
            environ_buf_offset += name_length;
            data[environ_buf_offset] = b'=';
            environ_buf_offset += 1;
            data[environ_buf_offset..environ_buf_offset + value_length].copy_from_slice(value);
            environ_buf_offset += value_length;
            data[environ_buf_offset] = 0;
            environ_buf_offset += 1;
        }

        Ok(())
    }
}

pub fn environ_sizes_get(
    mut caller: Caller<GlobalStore>,
    offset_0: i32,
    offset_1: i32,
) -> Result<i32, wasmi::Error> {
    let memory = get_memory(caller)?;

    let data = memory.data_mut(&mut caller);
    let environment_variables =
        block_on(task::get_instance().get_environment_variables(caller.data().wasi.task))
            .map_err(|_| wasmi::Error::new("Failed to get environment variables"))?;

    wasi_result! {
        let count = environment_variables.len() as i32;
        let total_size: i32 = environment_variables.iter()
            .map(|var| var.get_name().len() + 1 + var.get_value().len() + 1)
            .sum::<usize>() as i32;

    data[offset_0 as usize..offset_0 as usize + 4].copy_from_slice(&count.to_le_bytes());


        Ok(())
    }
}

}
