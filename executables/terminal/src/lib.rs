#![no_std]

extern crate alloc;

xila::internationalization::include_translations!();

mod device;
mod error;
mod executable;
mod terminal;

pub use executable::*;

use core::{num::NonZeroUsize, time::Duration};

use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};
use xila::executable::Standard;
use xila::file_system::{Device, Flags, Mode, UniqueFileIdentifier};
use xila::task::{self, TaskIdentifier};
use xila::virtual_file_system;

use crate::{error::Result, terminal::Terminal};

pub const SHORTCUT: &str = r#"
{
    "name": "Terminal",
    "command": "/binaries/terminal",
    "arguments": [],
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
        .mount_device(task, &"/devices/terminal", Device::new(terminal))
        .await?;

    let standard_in = virtual_file_system::get_instance()
        .open(
            &"/devices/terminal",
            Flags::new(Mode::READ_ONLY, None, None),
            task,
        )
        .await?;

    let standard_out = virtual_file_system::get_instance()
        .open(&"/devices/terminal", Mode::WRITE_ONLY.into(), task)
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

    xila::executable::execute("/binaries/command_line_shell", vec![], standard, None).await?;

    while terminal.event_handler().await? {
        task::Manager::sleep(Duration::from_millis(10)).await;
    }

    Ok(())
}

pub async fn main(standard: Standard, _: Vec<String>) -> core::result::Result<(), NonZeroUsize> {
    if let Err(error) = inner_main(standard.get_task()).await {
        standard.print_error(&error.to_string()).await;
        return Err(error.into());
    }

    Ok(())
}
