use core::ffi::c_int;
use std::mem::forget;

use crate::{Device::Device_type, Error_type, Position_type};

use super::littlefs;

pub unsafe extern "C" fn Read_callback(
    Configuration: *const littlefs::lfs_config,
    Block: littlefs::lfs_block_t,
    Offset: littlefs::lfs_off_t,
    Buffer: *mut core::ffi::c_void,
    Size: littlefs::lfs_size_t,
) -> c_int {
    let Device = unsafe { Box::from_raw(Configuration.read().context as *mut Device_type) };

    let Buffer = unsafe { core::slice::from_raw_parts_mut(Buffer as *mut u8, Size as usize) };

    let Block_size = unsafe { Configuration.read().block_size };

    let Position = Block as u64 * Block_size as u64 + Offset as u64;

    loop {
        match Device.Set_position(&Position_type::Start(Position)) {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    loop {
        match Device.Read(Buffer) {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    forget(Device);

    0
}

pub unsafe extern "C" fn Programm_callback(
    Configuration: *const littlefs::lfs_config,
    Block: littlefs::lfs_block_t,
    Offset: littlefs::lfs_off_t,
    Buffer: *const core::ffi::c_void,
    Size: littlefs::lfs_size_t,
) -> c_int {
    let Device = unsafe { Box::from_raw(Configuration.read().context as *mut Device_type) };

    let Buffer = unsafe { core::slice::from_raw_parts(Buffer as *const u8, Size as usize) };

    let Block_size = unsafe { Configuration.read().block_size };

    let Position = Block as u64 * Block_size as u64 + Offset as u64;

    loop {
        match Device.Set_position(&Position_type::Start(Position)) {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    loop {
        match Device.Write(Buffer) {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    forget(Device);

    0
}

pub unsafe extern "C" fn Erase_callback(
    Configuration: *const littlefs::lfs_config,
    Block: littlefs::lfs_block_t,
) -> c_int {
    let Device = unsafe { Box::from_raw(Configuration.read().context as *mut Device_type) };

    let Block_size = unsafe { Configuration.read().block_size };

    let Position = Block as u64 * Block_size as u64;

    loop {
        match Device.Set_position(&Position_type::Start(Position)) {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    loop {
        match Device.Erase() {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    forget(Device);

    0
}

pub unsafe extern "C" fn Flush_callback(Configuration: *const littlefs::lfs_config) -> c_int {
    let Device = unsafe { Box::from_raw(Configuration.read().context as *mut Device_type) };

    loop {
        match Device.Flush() {
            Ok(_) => break,
            Err(Error_type::Ressource_busy) => continue,
            Err(_) => return littlefs::lfs_error_LFS_ERR_IO,
        }
    }

    forget(Device);

    0
}
