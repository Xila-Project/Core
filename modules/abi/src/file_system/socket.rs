use super::{into_u32, XilaFileSystemResult, XilaUniqueFileIdentifier};
use futures::block_on;
use task::get_instance as get_task_manager_instance;
use virtual_file_system::{get_instance as get_file_system_instance, Error};

/// This function is used to send data through a socket.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn xila_file_system_send(
    socket: XilaUniqueFileIdentifier,
    buffer: *const u8,
    size: usize,
) -> XilaFileSystemResult {
    into_u32(|| {
        let task = block_on(get_task_manager_instance().get_current_task_identifier());

        let socket = file_system::UniqueFileIdentifier::from_raw(socket);

        if buffer.is_null() {
            Err(Error::InvalidParameter)?;
        }

        let buffer = core::slice::from_raw_parts(buffer, size);

        block_on(get_file_system_instance().send(task, socket, buffer))?;

        Ok(())
    })
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
#[no_mangle]
pub unsafe extern "C" fn xila_file_system_receive(
    socket: XilaUniqueFileIdentifier,
    buffer: *mut u8,
    size: usize,
) -> XilaFileSystemResult {
    into_u32(|| {
        let task = block_on(get_task_manager_instance().get_current_task_identifier());

        let socket = file_system::UniqueFileIdentifier::from_raw(socket);

        if buffer.is_null() {
            Err(Error::InvalidParameter)?;
        }

        let buffer = core::slice::from_raw_parts_mut(buffer, size);

        block_on(get_file_system_instance().receive(task, socket, buffer))?;

        Ok(())
    })
}
