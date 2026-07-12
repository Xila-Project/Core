use wasmi::Caller;
use xila::task::{self, block_on};

use crate::{
    define_wasi_module,
    host::{store::GlobalStore, wasi::memory::WasmMemory},
    wasi_result,
};

fn write(memory: &mut WasmMemory, offset: usize, bytes: &[u8]) {
    memory.write(offset, bytes).ok();
}

define_wasi_module! {
    module: "wasi_snapshot_preview1";

    fn args_get(
        caller: Caller<GlobalStore>,
        argc_ptr: i32,
        argv_buf_size_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        let mut caller = caller;
        let arguments = caller.data().wasi.args.clone();
        let mut memory = WasmMemory::from_caller(&mut caller)?;

        wasi_result! {
            let mut argv_offset = argc_ptr as usize;
            let mut buf_offset = argv_buf_size_ptr as usize;

            for arg in &arguments {
                write(&mut memory, argv_offset, &(buf_offset as i32).to_le_bytes());
                argv_offset += 4;
                write(&mut memory, buf_offset, arg);
                buf_offset += arg.len();
                write(&mut memory, buf_offset, &[0u8; 1]);
                buf_offset += 1;
            }

            Ok(())
        }
    }

    fn args_sizes_get(
        caller: Caller<GlobalStore>,
        argc_ptr: i32,
        argv_buf_size_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        let mut caller = caller;
        let arguments = caller.data().wasi.args.clone();
        let mut memory = WasmMemory::from_caller(&mut caller)?;

        wasi_result! {
            let argc = arguments.len() as i32;
            let argv_buf_size = arguments.iter().map(|a| a.len() + 1).sum::<usize>() as i32;

            write(&mut memory, argc_ptr as usize, &argc.to_le_bytes());
            write(&mut memory, argv_buf_size_ptr as usize, &argv_buf_size.to_le_bytes());

            Ok(())
        }
    }

    fn environ_get(
        caller: Caller<GlobalStore>,
        environ: i32,
        environ_buf: i32,
    ) -> Result<i32, wasmi::Error> {
        xila::log::information!(
            "environ_get: pointers={:#x}, buffer={:#x}",
            environ, environ_buf
        );
        let mut caller = caller;
        let task = caller.data().wasi.task;
        let mut memory = WasmMemory::from_caller(&mut caller)?;
        let environment_variables =
            block_on(task::get_instance().get_environment_variables(task))
                .map_err(|_| wasmi::Error::new("Failed to get environment variables"))?;
        xila::log::information!("environ_get: {} variables", environment_variables.len());

        wasi_result! {
            let mut environ_offset = environ as usize;
            let mut environ_buf_offset = environ_buf as usize;

            for variable in &environment_variables {
                write(&mut memory, environ_offset, &(environ_buf_offset as i32).to_le_bytes());
                environ_offset += 4;
                write(&mut memory, environ_buf_offset, variable.get_name().as_bytes());
                environ_buf_offset += variable.get_name().len();
                write(&mut memory, environ_buf_offset, b"=");
                environ_buf_offset += 1;
                write(&mut memory, environ_buf_offset, variable.get_value().as_bytes());
                environ_buf_offset += variable.get_value().len();
                write(&mut memory, environ_buf_offset, &[0u8; 1]);
                environ_buf_offset += 1;
            }

            xila::log::information!(
                "environ_get: wrote pointers through {:#x}, buffer through {:#x}",
                environ_offset, environ_buf_offset
            );
            Ok(())
        }
    }

    fn environ_sizes_get(
        caller: Caller<GlobalStore>,
        environ_count_ptr: i32,
        environ_buf_size_ptr: i32,
    ) -> Result<i32, wasmi::Error> {
        let mut caller = caller;
        let task = caller.data().wasi.task;
        let mut memory = WasmMemory::from_caller(&mut caller)?;
        let environment_variables =
            block_on(task::get_instance().get_environment_variables(task))
                .map_err(|_| wasmi::Error::new("Failed to get environment variables"))?;

        wasi_result! {
            let count = environment_variables.len() as i32;
            let total_size = environment_variables.iter()
                .map(|var| var.get_name().len() + 1 + var.get_value().len() + 1)
                .sum::<usize>() as i32;

            write(&mut memory, environ_count_ptr as usize, &count.to_le_bytes());
            write(&mut memory, environ_buf_size_ptr as usize, &total_size.to_le_bytes());

            Ok(())
        }
    }
}
