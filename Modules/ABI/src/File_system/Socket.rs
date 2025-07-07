use super::{Into_u32, Xila_file_system_result_type, Xila_unique_file_identifier_type};
use Futures::block_on;
use Task::Get_instance as Get_task_manager_instance;
use Virtual_file_system::{Error_type, Get_instance as Get_file_system_instance};

/// This function is used to send data through a socket.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_send(
    socket: Xila_unique_file_identifier_type,
    buffer: *const u8,
    size: usize,
) -> Xila_file_system_result_type {
    Into_u32(|| {
        let task = block_on(Get_task_manager_instance().Get_current_task_identifier());

        let Socket = File_system::Unique_file_identifier_type::From_raw(socket);

        if buffer.is_null() {
            Err(Error_type::Invalid_parameter)?;
        }

        let Buffer = core::slice::from_raw_parts(buffer, size);

        block_on(Get_file_system_instance().Send(task, Socket, Buffer))?;

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
pub unsafe extern "C" fn Xila_file_system_receive(
    socket: Xila_unique_file_identifier_type,
    buffer: *mut u8,
    size: usize,
) -> Xila_file_system_result_type {
    Into_u32(|| {
        let task = block_on(Get_task_manager_instance().Get_current_task_identifier());

        let Socket = File_system::Unique_file_identifier_type::From_raw(socket);

        if buffer.is_null() {
            Err(Error_type::Invalid_parameter)?;
        }

        let Buffer = core::slice::from_raw_parts_mut(buffer, size);

        block_on(Get_file_system_instance().Receive(task, Socket, Buffer))?;

        Ok(())
    })
}
