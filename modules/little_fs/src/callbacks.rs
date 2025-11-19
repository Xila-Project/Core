use core::ffi::{c_int, c_void};

use alloc::slice;
use file_system::Error;

use crate::configuration::Context;

use super::littlefs;

pub unsafe extern "C" fn read_callback(
    configuration: *const littlefs::lfs_config,
    block: littlefs::lfs_block_t,
    offset: littlefs::lfs_off_t,
    buffer: *mut c_void,
    size: littlefs::lfs_size_t,
) -> c_int {
    let context = unsafe { Context::get_from_configuration(configuration) };

    let buffer = unsafe { slice::from_raw_parts_mut(buffer as *mut u8, size as usize) };

    let block_size = unsafe { configuration.read().block_size };

    let position = block as u64 * block_size as u64 + offset as u64;

    loop {
        match context.device.read(buffer, position) {
            Ok(_) => break,
            Err(Error::RessourceBusy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    0
}

pub unsafe extern "C" fn programm_callback(
    configuration: *const littlefs::lfs_config,
    block: littlefs::lfs_block_t,
    offset: littlefs::lfs_off_t,
    buffer: *const c_void,
    size: littlefs::lfs_size_t,
) -> c_int {
    let context = unsafe { Context::get_from_configuration(configuration) };
    let buffer = unsafe { slice::from_raw_parts(buffer as *const u8, size as usize) };

    let block_size = unsafe { configuration.read().block_size };

    let position = block as u64 * block_size as u64 + offset as u64;

    loop {
        match context.device.write(buffer, position) {
            Ok(_) => break,
            Err(Error::RessourceBusy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    0
}

pub unsafe extern "C" fn erase_callback(
    configuration: *const littlefs::lfs_config,
    block: littlefs::lfs_block_t,
) -> c_int {
    let context = unsafe { Context::get_from_configuration(configuration) };
    let block_size = unsafe { configuration.read().block_size };

    let position = block as u64 * block_size as u64;

    loop {
        match context
            .device
            .write_pattern(&[0x00], block_size as usize, position)
        {
            Ok(_) => break,
            Err(Error::RessourceBusy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    0
}

pub unsafe extern "C" fn flush_callback(configuration: *const littlefs::lfs_config) -> c_int {
    let context = unsafe { Context::get_from_configuration(configuration) };
    loop {
        match context.device.flush() {
            Ok(_) => break,
            Err(Error::RessourceBusy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    0
}
