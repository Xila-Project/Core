use alloc::{string::String, vec::Vec};
use wasmi::Caller;
use xila::task::{self, block_on};

use crate::{
    define_wasi_module,
    host::{
        store::GlobalStore,
        translation::{FromGuest, GuestPointer, GuestSlice, WasmAdress, WasmUsize, get_memory},
        wasi::{
            Error,
            error::{WasiResult, wrap_function},
        },
    },
};

fn count_arguments_sizes(caller: &Vec<String>) -> (WasmUsize, WasmUsize) {
    let argc = caller.len() as WasmUsize;
    let argv_buf_size = caller.iter().map(|arg| arg.len() + 1).sum::<usize>() as WasmUsize;
    (argc, argv_buf_size)
}

define_wasi_module! {
    module: "wasi_snapshot_preview1";

    fn args_get(
        caller: Caller<GlobalStore>,
        argv: WasmAdress,
        argv_buf: WasmAdress,
    ) -> Result<WasiResult, wasmi::Error> {
        wrap_function!({
            let mut caller = caller;

            let (argc, argv_buf_size) = count_arguments_sizes(&caller.data().wasi.arguments);
            let memory = get_memory(&mut caller).ok_or(Error::Fault)?;



            let argv : *mut [GuestPointer<u8>] = GuestSlice::new(argv, argc).from_guest(memory).ok_or(Error::Fault)?;
            let argv_buf : *mut [u8] = GuestSlice::new(argv_buf, argv_buf_size).from_guest(memory).ok_or(Error::Fault)?;

            let mut argv_buf_offset = 0;

            for (arg, arg_str) in unsafe { &mut *argv }.iter_mut().zip(caller.data().wasi.arguments.iter()) {
                let arg_bytes = arg_str.as_bytes();
                let arg_len = arg_bytes.len();

                if argv_buf_offset + arg_len + 1 > argv_buf_size as _ {
                    return Err(Error::Fault.into());
                }

                unsafe {
                    *arg = GuestPointer::new(argv_buf_offset as WasmUsize);
                    let arg_buf = &mut (*argv_buf)[argv_buf_offset..argv_buf_offset + arg_len];
                    arg_buf.copy_from_slice(arg_bytes);
                    (*argv_buf)[argv_buf_offset + arg_len] = 0; // Null-terminate
                }
                argv_buf_offset += arg_len + 1;

            }


            Ok(())
        })
    }

    fn args_sizes_get(
        caller: Caller<GlobalStore>,
        argc_ptr: WasmAdress,
        argv_buf_size_ptr: WasmAdress,
    ) -> Result<WasiResult, wasmi::Error> {
        wrap_function!({
            let mut caller = caller;
            let memory = get_memory(&mut caller).ok_or(Error::Fault)?;

            let argc : *mut WasmUsize = GuestPointer::new(argc_ptr).from_guest(memory).ok_or(Error::Fault)?;
            let argv_buf_size : *mut WasmUsize = GuestPointer::new(argv_buf_size_ptr).from_guest(memory).ok_or(Error::Fault)?;

            let result = count_arguments_sizes(&caller.data().wasi.arguments);

            unsafe {
                *argc = result.0;
                *argv_buf_size = result.1;
            }

            Ok(())
        })
    }

    fn environ_get(
        caller: Caller<GlobalStore>,
        environ: i32,
        environ_buf: i32,
    ) -> Result<i32, wasmi::Error> {
        wrap_function!({
            let mut caller = caller;

            let task = caller.data().wasi.task;

            let environment_variables =
                block_on(task::get_instance().get_environment_variables(task))
                .map_err(|_| wasmi::Error::new("Failed to get environment variables"))?;

            let memory = get_memory(&mut caller).ok_or(Error::Fault)?;

            let environ : *mut [GuestPointer<u8>] = GuestSlice::new(environ as WasmUsize, environment_variables.len() as WasmUsize).from_guest(memory).ok_or(Error::Fault)?;
            let environ_buf : *mut [u8] = GuestSlice::new(environ_buf as WasmUsize, environment_variables.iter().map(|var| var.get_name().len() + 1 + var.get_value().len() + 1).sum::<usize>() as WasmUsize).from_guest(memory).ok_or(Error::Fault)?;

            let mut environ_buf_offset = 0;

            for (env_var, env_var_str) in unsafe { &mut *environ }.iter_mut().zip(environment_variables.iter()) {
                // do not use format!() here to avoid heap allocation, instead just use an offset into the buffer and copy the bytes directly
                let mut env_var_offset = environ_buf_offset;



                let env_var_bytes = format!("{}={}", env_var_str.get_name(), env_var_str.get_value()).as_bytes();
                let env_var_len = env_var_bytes.len();

                if environ_buf_offset + env_var_len + 1 > environ_buf.len() {
                    return Err(Error::Fault.into());
                }

                unsafe {
                    *env_var = GuestPointer::new(environ_buf_offset as WasmUsize);
                    let env_var_buf = &mut (*environ_buf)[environ_buf_offset..environ_buf_offset + env_var_len];
                    env_var_buf.copy_from_slice(env_var_bytes);
                    (*environ_buf)[environ_buf_offset + env_var_len] = 0; // Null-terminate
                }
                environ_buf_offset += env_var_len + 1;
            }

            Ok(())
        })
    }

    fn environ_sizes_get(
        caller: Caller<GlobalStore>,
        environ_count_ptr: WasmAdress,
        environ_buf_size_ptr: WasmAdress,
    ) -> Result<WasiResult, wasmi::Error> {
        wrap_function!({
            let mut caller = caller;
            let task = caller.data().wasi.task;

            let environment_variables =
                block_on(task::get_instance().get_environment_variables(task))
                .map_err(|_| wasmi::Error::new("Failed to get environment variables"))?;

            let memory = get_memory(&mut caller).ok_or(Error::Fault)?;

            let environ_count : *mut WasmUsize = GuestPointer::new(environ_count_ptr).from_guest(memory).ok_or(Error::Fault)?;
            let environ_buf_size : *mut WasmUsize = GuestPointer::new(environ_buf_size_ptr).from_guest(memory).ok_or(Error::Fault)?;


            unsafe {
                *environ_count = environment_variables.len() as WasmUsize;
                *environ_buf_size = environment_variables.iter().map(|var| var.get_name().len() + 1 + var.get_value().len() + 1).sum::<usize>() as WasmUsize;
            }

            Ok(())
        })
    }
}
