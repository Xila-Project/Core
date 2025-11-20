use core::{
    alloc::Layout,
    ffi::{c_char, c_void},
    ptr::null_mut,
    slice,
};

#[unsafe(no_mangle)]
pub extern "C" fn lfs_crc(mut crc: u32, data: *const c_void, size: usize) -> u32 {
    const RTABLE: [u32; 16] = [
        0x00000000, 0x1db71064, 0x3b6e20c8, 0x26d930ac, 0x76dc4190, 0x6b6b51f4, 0x4db26158,
        0x5005713c, 0xedb88320, 0xf00f9344, 0xd6d6a3e8, 0xcb61b38c, 0x9b64c2b0, 0x86d3d2d4,
        0xa00ae278, 0xbdbdf21c,
    ];

    if data.is_null() {
        return crc;
    }

    let data = unsafe { slice::from_raw_parts(data as *const u8, size) };

    for &byte in data {
        let byte = byte as u32;
        crc = (crc >> 4) ^ RTABLE[((crc ^ byte) & 0xf) as usize];
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lfs_malloc(size: usize) -> *mut c_void {
    if size == 0 {
        return null_mut();
    }

    let total_size = size + size_of::<usize>();
    let alignment = align_of::<usize>();

    let layout = Layout::from_size_align(total_size, alignment).unwrap();

    unsafe {
        let ptr = alloc::alloc::alloc(layout);

        if ptr.is_null() {
            return null_mut();
        }

        // Store the size at the beginning
        let size_ptr = ptr as *mut usize;
        *size_ptr = size;

        // Return pointer to the memory after the size
        ptr.add(size_of::<usize>()) as *mut c_void
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lfs_free(p: *mut c_void) {
    if p.is_null() {
        return;
    }

    unsafe {
        // Get the original pointer by subtracting the size of usize
        let original_ptr = (p as *mut u8).sub(size_of::<usize>());

        // Read the size stored at the beginning
        let size_ptr = original_ptr as *mut usize;
        let size = *size_ptr;

        let total_size = size + size_of::<usize>();
        let alignment = align_of::<usize>();

        let layout = Layout::from_size_align(total_size, alignment).unwrap();

        alloc::alloc::dealloc(original_ptr, layout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfs_malloc_and_free() {
        unsafe {
            let size = 100;
            let ptr = lfs_malloc(size);
            assert!(!ptr.is_null());

            // Write some data
            let data_ptr = ptr as *mut u8;
            for i in 0..size {
                *data_ptr.add(i) = i as u8;
            }

            // Verify the data
            for i in 0..size {
                assert_eq!(*data_ptr.add(i), i as u8);
            }

            lfs_free(ptr);
        }
    }
}
