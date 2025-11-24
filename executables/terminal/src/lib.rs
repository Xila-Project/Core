#![no_std]

extern crate alloc;

mod device;
mod error;
mod executable;
mod terminal;

use alloc::boxed::Box;
pub use executable::*;

use alloc::{string::String, vec, vec::Vec};
use core::fmt::Write;
use core::{num::NonZeroUsize, time::Duration};
use error::*;
use xila::executable::Standard;
use xila::log;
use xila::task::{self, TaskIdentifier};
use xila::virtual_file_system::{self, ItemStatic};

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

async fn mount_and_open(task: TaskIdentifier, terminal: &'static Terminal) -> Result<Standard> {
    let virtual_file_system = virtual_file_system::get_instance();

    let _ = virtual_file_system.remove(task, &"/devices/terminal").await;

    virtual_file_system
        .mount_static(
            task,
            &"/devices/terminal",
            ItemStatic::CharacterDevice(terminal),
        )
        .await?;

    let standard = Standard::open(
        &"/devices/terminal",
        &"/devices/terminal",
        &"/devices/terminal",
        task,
        virtual_file_system,
    )
    .await?;

    Ok(standard)
}

async fn inner_main(task: TaskIdentifier) -> Result<()> {
    let terminal = Terminal::new().await?;

    let terminal = Box::leak(Box::new(terminal));

    let standard = mount_and_open(task, terminal).await?;

    xila::executable::execute("/binaries/command_line_shell", vec![], standard, None).await?;

    while terminal.handle_events().await? {
        task::Manager::sleep(Duration::from_millis(20)).await;
    }

    Ok(())
}

pub async fn main(
    mut standard: Standard,
    _: Vec<String>,
) -> core::result::Result<(), NonZeroUsize> {
    let task = task::get_instance().get_current_task_identifier().await;

    if let Err(error) = inner_main(task).await {
        let _ = writeln!(standard.error(), "{}", error);
        log::error!("Terminal executable error: {}", error);
        return Err(error.into());
    }

    Ok(())
}
