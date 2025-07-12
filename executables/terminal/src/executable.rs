use crate::main;
use alloc::string::{String, ToString};
use executable::implement_executable_device;
use file_system::{Flags, Mode, Open};
use task::TaskIdentifier;
use virtual_file_system::{File, VirtualFileSystemType};

pub struct TerminalExecutableType;

impl TerminalExecutableType {
    pub async fn new<'a>(
        virtual_file_system: &'a VirtualFileSystemType<'a>,
        task: TaskIdentifier,
    ) -> Result<Self, String> {
        let _ = virtual_file_system
            .create_directory(&"/Configuration/Shared/Shortcuts", task)
            .await;

        let file = match File::open(
            virtual_file_system,
            "/Configuration/Shared/Shortcuts/Terminal.json",
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
    Structure: TerminalExecutableType,
    Mount_path: "/Binaries/Terminal",
    Main_function: main,
);
