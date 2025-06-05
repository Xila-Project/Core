use core::num::NonZeroUsize;
use core::time::Duration;

use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use Executable::Standard_type;
use File_system::{Device_type, Flags_type, Mode_type, Unique_file_identifier_type};
use Task::Task_identifier_type;

use crate::{Error::Result_type, Terminal::Terminal_type};

async fn Mount_and_open(
    Task: Task_identifier_type,
    Terminal: Arc<Terminal_type>,
) -> Result_type<(
    Unique_file_identifier_type,
    Unique_file_identifier_type,
    Unique_file_identifier_type,
)> {
    Virtual_file_system::Get_instance()
        .Mount_device(Task, &"/Devices/Terminal", Device_type::New(Terminal))
        .await?;

    let Standard_in = Virtual_file_system::Get_instance()
        .Open(
            &"/Devices/Terminal",
            Flags_type::New(Mode_type::Read_only, None, None),
            Task,
        )
        .await?;

    let Standard_out = Virtual_file_system::Get_instance()
        .Open(&"/Devices/Terminal", Mode_type::Write_only.into(), Task)
        .await?;

    let Standard_error = Virtual_file_system::Get_instance()
        .Duplicate_file_identifier(Standard_out, Task)
        .await?;

    Ok((Standard_in, Standard_out, Standard_error))
}

async fn Inner_main(Task: Task_identifier_type) -> Result_type<()> {
    let Terminal = Terminal_type::New().await?;

    let Terminal = Arc::new(Terminal);

    let (Standard_in, Standard_out, Standard_error) =
        Mount_and_open(Task, Terminal.clone()).await?;

    let Standard = Standard_type::New(
        Standard_in,
        Standard_out,
        Standard_error,
        Task::Get_instance().Get_current_task_identifier().await,
        Virtual_file_system::Get_instance(),
    );

    Executable::Execute("/Binaries/Command_line_shell", "".to_string(), Standard).await?;

    while Terminal.Event_handler().await? {
        Task::Manager_type::Sleep(Duration::from_millis(20)).await;
    }

    Ok(())
}

pub async fn Main(Standard: Standard_type, _: String) -> Result<(), NonZeroUsize> {
    if let Err(Error) = Inner_main(Standard.Get_task()).await {
        Standard.Print_error(&Error.to_string()).await;
        return Err(Error.into());
    }

    Ok(())
}
