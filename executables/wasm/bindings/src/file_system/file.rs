use crate::FileVariantKind;
use crate::host::file_system::{FileSystemItem, XilaFileSystemItem};
use xila::abi_declarations::{
    XilaFileSystemResult, XilaFileSystemSize, XilaFileSystemState, xila_file_system_file_close,
    xila_file_system_file_flush, xila_file_system_file_is_a_terminal, xila_file_system_file_read,
    xila_file_system_file_read_at, xila_file_system_file_set_flags, xila_file_system_file_write,
    xila_file_system_file_write_at,
};
use xila::{log, virtual_file_system};

macro_rules! with_file {
    ($ptr:expr, $file_var:ident => $body:expr) => {
        unsafe {
            match FileSystemItem::borrow_from_raw($ptr as _) {
                FileSystemItem::File($file_var) => $body,
                _ => XilaFileSystemResult::from(virtual_file_system::Error::InvalidParameter),
            }
        }
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_file_close(
    item: *mut XilaFileSystemItem,
) -> XilaFileSystemResult {
    log::information!("Closing file {:?}", item);
    unsafe {
        let file = FileSystemItem::own_from_raw(item as _);

        match *file {
            FileSystemItem::File(mut file) => xila_file_system_file_close(&mut file.file as _),
            _ => virtual_file_system::Error::InvalidParameter.into(),
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_file_write(
    item: *mut XilaFileSystemItem,
    buffers: *const *const u8,
    buffer_lengths: *const usize,
    buffer_count: usize,
    written: *mut usize,
) -> XilaFileSystemResult {
    log::information!("Writing to file {:?} ", item);
    with_file!(item, f => xila_file_system_file_write(
        &mut f.file,
        buffers,
        buffer_lengths,
        buffer_count,
        written
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_file_read(
    item: *mut XilaFileSystemItem,
    buffers: *const *mut u8,
    buffer_lengths: *const usize,
    buffer_count: usize,
    read: *mut usize,
) -> XilaFileSystemResult {
    log::information!("Reading from file {:?} ", item);
    with_file!(item, f => xila_file_system_file_read(
        &mut f.file,
        buffers,
        buffer_lengths,
        buffer_count,
        read
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_file_write_at(
    item: *mut XilaFileSystemItem,
    offset: XilaFileSystemSize,
    buffers: *const *const u8,
    buffer_lengths: *const usize,
    buffer_count: usize,
    written: *mut usize,
) -> XilaFileSystemResult {
    log::information!("Writing to file {:?} at offset {} ", item, offset);
    with_file!(item, f => xila_file_system_file_write_at(
        &mut f.file,
        offset,
        buffers,
        buffer_lengths,
        buffer_count,
        written
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_file_read_at(
    item: *mut XilaFileSystemItem,
    offset: XilaFileSystemSize,
    buffers: *const *mut u8,
    buffer_lengths: *const usize,
    buffer_count: usize,
    read: *mut usize,
) -> XilaFileSystemResult {
    log::information!("Reading from file {:?} at offset {} ", item, offset);
    with_file!(item, f => xila_file_system_file_read_at(
        &mut f.file,
        offset,
        buffers,
        buffer_lengths,
        buffer_count,
        read
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_file_is_a_terminal(
    item: *mut XilaFileSystemItem,
    is_terminal: *mut bool,
) -> XilaFileSystemResult {
    log::information!("Checking if file {:?} is a terminal ", item);
    with_file!(item, f => xila_file_system_file_is_a_terminal(
        &mut f.file,
        is_terminal
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_file_is_standard_input(
    item: *mut XilaFileSystemItem,
) -> bool {
    log::information!("Checking if file {:?} is standard input ", item);
    with_file!(item, f => {
        return f.kind == FileVariantKind::StandardInput;
    });

    return false;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_file_is_standard_output(
    item: *mut XilaFileSystemItem,
) -> bool {
    log::information!("Checking if file {:?} is standard output ", item);
    with_file!(item, f => {
           return f.kind == FileVariantKind::StandardOutput;
    });

    return false;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_file_is_standard_error(
    item: *mut XilaFileSystemItem,
) -> bool {
    log::information!("Checking if file {:?} is standard error ", item);
    with_file!(item, f => {
         return f.kind == FileVariantKind::StandardError;
    });

    return false;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_file_set_flags(
    item: *mut XilaFileSystemItem,
    state: XilaFileSystemState,
) -> XilaFileSystemResult {
    log::information!("Setting flags for file {:?} to {:?} ", item, state);
    with_file!(item, f => xila_file_system_file_set_flags(
        &mut f.file,
        state
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_file_flush(
    item: *mut XilaFileSystemItem,
    flush_data: bool,
) -> XilaFileSystemResult {
    log::information!("Flushing file {:?} ", item);
    with_file!(item, f => xila_file_system_file_flush(
        &mut f.file,
        flush_data
    ))
}
