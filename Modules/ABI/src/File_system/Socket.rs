use super::{Into_u32, Xila_file_system_result_type, Xila_unique_file_identifier_type};
use Task::Get_instance as Get_task_manager_instance;
use Virtual_file_system::{Error_type, Get_instance as Get_file_system_instance};

/// This function is used to send data through a socket.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_send(
    Socket: Xila_unique_file_identifier_type,
    Buffer: *const u8,
    Size: usize,
) -> Xila_file_system_result_type {
    Into_u32(|| {
        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Socket = File_system::Unique_file_identifier_type::From_raw(Socket);

        if Buffer.is_null() {
            Err(Error_type::Invalid_parameter)?;
        }

        let Buffer = std::slice::from_raw_parts(Buffer, Size);

        Get_file_system_instance().Send(Task, Socket, Buffer)?;

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
    Socket: Xila_unique_file_identifier_type,
    Buffer: *mut u8,
    Size: usize,
) -> Xila_file_system_result_type {
    Into_u32(|| {
        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Socket = File_system::Unique_file_identifier_type::From_raw(Socket);

        if Buffer.is_null() {
            Err(Error_type::Invalid_parameter)?;
        }

        let Buffer = std::slice::from_raw_parts_mut(Buffer, Size);

        Get_file_system_instance().Receive(Task, Socket, Buffer)?;

        Ok(())
    })
}
