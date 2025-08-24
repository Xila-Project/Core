use core::ffi::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn lfs_crc(crc: u32, data: *const u8, size: usize) -> u32 {
    static RTABLE: [u32; 16] = [
        0x00000000, 0x1db71064, 0x3b6e20c8, 0x26d930ac, 0x76dc4190, 0x6b6b51f4, 0x4db26158,
        0x5005713c, 0xedb88320, 0xf00f9344, 0xd6d6a3e8, 0xcb61b38c, 0x9b64c2b0, 0x86d3d2d4,
        0xa00ae278, 0xbdbdf21c,
    ];

    let mut crc: u32 = crc;

    for i in 0..size {
        let byte = unsafe { *data.add(i) } as u32;
        crc = (crc >> 4) ^ RTABLE[((crc ^ (byte >> 0)) & 0xf) as usize];
        crc = (crc >> 4) ^ RTABLE[((crc ^ (byte >> 4)) & 0xf) as usize];
    }

    crc
}

#[unsafe(no_mangle)]
pub extern "C" fn strcpy(destination: *mut c_char, source: *const c_char) -> *mut c_char {
    if destination.is_null() || source.is_null() {
        return destination;
    }

    let mut dst_ptr = destination;
    let mut src_ptr = source;

    unsafe {
        // Copy characters until null terminator
        while *src_ptr != 0 {
            *dst_ptr = *src_ptr;
            dst_ptr = dst_ptr.add(1);
            src_ptr = src_ptr.add(1);
        }

        // Null-terminate the destination string
        *dst_ptr = 0;
    }

    destination
}
