#![no_std]

extern crate alloc;

mod device;
mod error;
mod executable;
mod terminal;

pub use executable::*;

use core::num::NonZeroUsize;

use ::executable::Standard;
use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use file_system::{DeviceType, Flags, Mode, UniqueFileIdentifier};
use futures::yield_now;
use task::TaskIdentifier;

use crate::{error::Result, terminal::Terminal};

pub const SHORTCUT: &str = r#"
{
    "name": "Terminal",
    "command": "/Binaries/Terminal",
    "arguments": "",
    "terminal": false,
    "icon_string": ">_",
    "icon_color": [0, 0, 0]
}"#;

async fn mount_and_open(
    task: TaskIdentifier,
    terminal: Arc<Terminal>,
) -> Result<(
    UniqueFileIdentifier,
    UniqueFileIdentifier,
    UniqueFileIdentifier,
)> {
    virtual_file_system::get_instance()
        .mount_device(task, &"/Devices/Terminal", DeviceType::new(terminal))
        .await?;

    let standard_in = virtual_file_system::get_instance()
        .open(
            &"/Devices/Terminal",
            Flags::new(Mode::READ_ONLY, None, None),
            task,
        )
        .await?;

    let standard_out = virtual_file_system::get_instance()
        .open(&"/Devices/Terminal", Mode::WRITE_ONLY.into(), task)
        .await?;

    let standard_error = virtual_file_system::get_instance()
        .duplicate_file_identifier(standard_out, task)
        .await?;

    Ok((standard_in, standard_out, standard_error))
}

async fn inner_main(task: TaskIdentifier) -> Result<()> {
    let terminal = Terminal::new().await?;

    let terminal: Arc<Terminal> = Arc::new(terminal);

    let (standard_in, standard_out, standard_error) =
        mount_and_open(task, terminal.clone()).await?;

    let standard = Standard::new(
        standard_in,
        standard_out,
        standard_error,
        task::get_instance().get_current_task_identifier().await,
        virtual_file_system::get_instance(),
    );

    ::executable::execute("/Binaries/Command_line_shell", "".to_string(), standard).await?;

    while terminal.event_handler().await? {
        yield_now().await;
    }

    Ok(())
}

pub async fn main(standard: Standard, _: String) -> core::result::Result<(), NonZeroUsize> {
    if let Err(error) = inner_main(standard.get_task()).await {
        standard.print_error(&error.to_string()).await;
        return Err(error.into());
    }

    Ok(())
}
