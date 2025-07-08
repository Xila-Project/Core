use core::num::NonZeroUsize;

use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use Executable::Standard_type;
use File_system::{Device_type, Flags_type, Mode_type, Unique_file_identifier_type};
use Futures::yield_now;
use Task::Task_identifier_type;

use crate::{terminal::Terminal_type, Error::Result_type};

async fn mount_and_open(
    task: Task_identifier_type,
    terminal: Arc<Terminal_type>,
) -> Result_type<(
    Unique_file_identifier_type,
    Unique_file_identifier_type,
    Unique_file_identifier_type,
)> {
    Virtual_file_system::get_instance()
        .mount_device(task, &"/Devices/Terminal", Device_type::New(terminal))
        .await?;

    let standard_in = Virtual_file_system::get_instance()
        .open(
            &"/Devices/Terminal",
            Flags_type::New(Mode_type::READ_ONLY, None, None),
            task,
        )
        .await?;

    let standard_out = Virtual_file_system::get_instance()
        .open(&"/Devices/Terminal", Mode_type::WRITE_ONLY.into(), task)
        .await?;

    let standard_error = Virtual_file_system::get_instance()
        .duplicate_file_identifier(standard_out, task)
        .await?;

    Ok((standard_in, standard_out, standard_error))
}

async fn inner_main(task: Task_identifier_type) -> Result_type<()> {
    let terminal = Terminal_type::new().await?;

    let terminal: Arc<Terminal_type> = Arc::new(terminal);

    let (standard_in, standard_out, standard_error) =
        mount_and_open(task, terminal.clone()).await?;

    let standard = Standard_type::new(
        standard_in,
        standard_out,
        standard_error,
        Task::get_instance().get_current_task_identifier().await,
        Virtual_file_system::get_instance(),
    );

    Executable::execute("/Binaries/Command_line_shell", "".to_string(), standard).await?;

    while terminal.event_handler().await? {
        yield_now().await;
    }

    Ok(())
}

pub async fn main(standard: Standard_type, _: String) -> Result<(), NonZeroUsize> {
    if let Err(error) = inner_main(standard.get_task()).await {
        standard.print_error(&error.to_string()).await;
        return Err(error.into());
    }

    Ok(())
}
