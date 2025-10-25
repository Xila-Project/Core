use crate::main;
use alloc::string::{String, ToString};
use xila::executable::implement_executable_device;
use xila::file_system::{self, Flags, Mode, Open};
use xila::task::TaskIdentifier;
use xila::virtual_file_system::{File, VirtualFileSystem};

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
    structure: TerminalExecutable,
    mount_path: "/binaries/terminal",
    main_function: main,
);
