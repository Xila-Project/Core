use crate::main;
use alloc::string::{String, ToString};
use executable::implement_executable_device;
use file_system::{Flags, Mode, Open};
use task::TaskIdentifier;
use virtual_file_system::{File, VirtualFileSystem};

pub struct TerminalExecutable;

impl TerminalExecutable {
    pub async fn new<'a>(
        virtual_file_system: &'a VirtualFileSystem<'a>,
        task: TaskIdentifier,
    ) -> Result<Self, String> {
        let _ = virtual_file_system
            .create_directory(&"/configuration/shared/shortcuts", task)
            .await;

        let file = match File::open(
            virtual_file_system,
            "/configuration/shared/shortcuts/terminal.json",
            Flags::new(Mode::WRITE_ONLY, Open::CREATE_ONLY.into(), None),
        )
        .await
        {
            Ok(file) => file,
            Err(file_system::Error::AlreadyExists) => {
                return Ok(Self);
            }
            Err(error) => Err(error.to_string())?,
        };

        file.write(crate::SHORTCUT.as_bytes())
            .await
            .map_err(|error| error.to_string())?;

        Ok(Self)
    }
}

implement_executable_device!(
    Structure: TerminalExecutable,
    Mount_path: "/binaries/terminal",
    Main_function: main,
);
