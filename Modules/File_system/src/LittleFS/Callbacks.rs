use core::ffi::c_int;

use super::littlefs;

pub extern "C" fn Read_callback(
    Configuration: *const littlefs::lfs_config,
    Block: littlefs::lfs_block_t,
    Offset: littlefs::lfs_off_t,
    Buffer: *mut core::ffi::c_void,
    Size: littlefs::lfs_size_t,
) -> c_int {
    0
}

pub extern "C" fn Programm_callback(
    Configuration: *const littlefs::lfs_config,
    Block: littlefs::lfs_block_t,
    Offset: littlefs::lfs_off_t,
    Buffer: *const core::ffi::c_void,
    Size: littlefs::lfs_size_t,
) -> c_int {
    0
}

pub extern "C" fn Erase_callback(
    Configuration: *const littlefs::lfs_config,
    Block: littlefs::lfs_block_t,
) -> c_int {
    0
}

pub extern "C" fn Flush_callback(Configuration: *const littlefs::lfs_config) -> c_int {
    0
}
