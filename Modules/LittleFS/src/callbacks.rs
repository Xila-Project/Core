use core::ffi::c_int;
use core::mem::forget;

use alloc::boxed::Box;
use file_system::{Device_type, Error_type, Position_type};

use super::littlefs;

pub unsafe extern "C" fn read_callback(
    configuration: *const littlefs::lfs_config,
    block: littlefs::lfs_block_t,
    offset: littlefs::lfs_off_t,
    buffer: *mut core::ffi::c_void,
    size: littlefs::lfs_size_t,
) -> c_int {
    let device = unsafe { Box::from_raw(configuration.read().context as *mut Device_type) };

    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer as *mut u8, size as usize) };

    let block_size = unsafe { configuration.read().block_size };

    let position = block as u64 * block_size as u64 + offset as u64;

    loop {
        match device.set_position(&Position_type::Start(position)) {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    loop {
        match device.read(buffer) {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    forget(device);

    0
}

pub unsafe extern "C" fn programm_callback(
    configuration: *const littlefs::lfs_config,
    block: littlefs::lfs_block_t,
    offset: littlefs::lfs_off_t,
    buffer: *const core::ffi::c_void,
    size: littlefs::lfs_size_t,
) -> c_int {
    let device = unsafe { Box::from_raw(configuration.read().context as *mut Device_type) };

    let buffer = unsafe { core::slice::from_raw_parts(buffer as *const u8, size as usize) };

    let block_size = unsafe { configuration.read().block_size };

    let position = block as u64 * block_size as u64 + offset as u64;

    loop {
        match device.set_position(&Position_type::Start(position)) {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    loop {
        match device.write(buffer) {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    forget(device);

    0
}

pub unsafe extern "C" fn erase_callback(
    configuration: *const littlefs::lfs_config,
    block: littlefs::lfs_block_t,
) -> c_int {
    let device = unsafe { Box::from_raw(configuration.read().context as *mut Device_type) };

    let block_size = unsafe { configuration.read().block_size };

    let position = block as u64 * block_size as u64;

    loop {
        match device.set_position(&Position_type::Start(position)) {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    loop {
        match device.erase() {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    forget(device);

    0
}

pub unsafe extern "C" fn flush_callback(configuration: *const littlefs::lfs_config) -> c_int {
    let device = unsafe { Box::from_raw(configuration.read().context as *mut Device_type) };

    loop {
        match device.flush() {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    forget(device);

    0
}
