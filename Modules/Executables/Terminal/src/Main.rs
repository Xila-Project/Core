use core::num::NonZeroUsize;
use std::{sync::Arc, time::Duration};

use Executable::Standard_type;
use File_system::{Device_type, Flags_type, Mode_type, Unique_file_identifier_type};
use Task::Task_identifier_type;

use crate::{Error::Result_type, Terminal::Terminal_type};

fn Mount_and_open(
    Task: Task_identifier_type,
    Terminal: Arc<Terminal_type>,
) -> Result_type<(
    Unique_file_identifier_type,
    Unique_file_identifier_type,
    Unique_file_identifier_type,
)> {
    Virtual_file_system::Get_instance().Mount_device(
        Task,
        &"/Devices/Terminal",
        Device_type::New(Terminal),
    )?;

    let Standard_in = Virtual_file_system::Get_instance().Open(
        &"/Devices/Terminal",
        Flags_type::New(Mode_type::Read_only, None, None),
        Task,
    )?;

    let Standard_out = Virtual_file_system::Get_instance().Open(
        &"/Devices/Terminal",
        Mode_type::Write_only.into(),
        Task,
    )?;

    let Standard_error =
        Virtual_file_system::Get_instance().Duplicate_file_identifier(Standard_out, Task)?;

    Ok((Standard_in, Standard_out, Standard_error))
}

fn Inner_main(Task: Task_identifier_type) -> Result_type<()> {
    let Terminal = Terminal_type::New()?;

    let Terminal = Arc::new(Terminal);

    let (Standard_in, Standard_out, Standard_error) = Mount_and_open(Task, Terminal.clone())?;

    let Standard = Standard_type::New(
        Standard_in,
        Standard_out,
        Standard_error,
        Task::Get_instance().Get_current_task_identifier()?,
        Virtual_file_system::Get_instance(),
    );

    Executable::Execute("/Shell", "".to_string(), Standard)?;

    while Terminal.Event_handler()? {
        Task::Manager_type::Sleep(Duration::from_millis(20));
    }

    Ok(())
}

pub fn Main(Standard: Standard_type, _: String) -> Result<(), NonZeroUsize> {
    if let Err(Error) = Inner_main(Standard.Get_task()) {
        Standard.Print_error(&Error.to_string());
        return Err(Error.into());
    }

    Ok(())
}
