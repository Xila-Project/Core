use task::block_on;

use crate::XilaFileIdentifier;

fn into_i32<F>(function: F) -> i32
where
    F: FnOnce() -> Result<(), network::Error>,
{
    match function() {
        Ok(()) => 0,
        Err(error) => {
            log::error!("Network operation failed: {}", error);

            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_network_socket_create(
    identifier: *mut XilaFileIdentifier,
    is_tcp: bool,
) -> i32 {
    into_i32(|| {
        let context = abi_context::get_instance();

        let task = context.get_current_task_identifier();

        let new_identifier = if is_tcp {
            let socket = block_on(network::get_instance().new_tcp_socket(2048, 2048, None))?;
            context
                .insert_tcp_socket(task, socket)
                .ok_or(network::Error::Other)
        } else {
            let socket =
                block_on(network::get_instance().new_udp_socket(2048, 2048, 16, 16, None))?;
            context.insert_udp_socket(task, socket)?
        };

        unsafe {
            *identifier = new_identifier.into_inner();
        }

        Ok(())
    })
}
