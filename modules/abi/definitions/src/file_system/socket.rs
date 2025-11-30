use super::{XilaFileIdentifier, XilaFileSystemResult, into_u32};
use task::block_on;
use task::get_instance as get_task_manager_instance;
use virtual_file_system::Error;

/// This function is used to send data through a socket.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_send(
    _socket: XilaFileIdentifier,
    buffer: *const u8,
    size: usize,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(|| {
            let _task = block_on(get_task_manager_instance().get_current_task_identifier());

            if buffer.is_null() {
                Err(Error::InvalidParameter)?;
            }

            let _buffer = core::slice::from_raw_parts(buffer, size);

            Ok(())
        })
    }
}

/// This function is used to receive data through a socket.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the file system fails to receive the data.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_receive(
    _socket: XilaFileIdentifier,
    buffer: *mut u8,
    size: usize,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(|| {
            let _task = block_on(get_task_manager_instance().get_current_task_identifier());

            if buffer.is_null() {
                Err(Error::InvalidParameter)?;
            }

            let _buffer = core::slice::from_raw_parts_mut(buffer, size);

            Ok(())
        })
    }
}
