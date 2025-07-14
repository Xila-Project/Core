use alloc::string::String;

use file_system::{FileIdentifier, Mode, Path, Size, UniqueFileIdentifier};
use futures::block_on;
use task::TaskIdentifier;
use virtual_file_system::VirtualFileSystem;

use crate::Result;

pub struct Standard {
    standard_in: UniqueFileIdentifier,
    standard_out: UniqueFileIdentifier,
    standard_error: UniqueFileIdentifier,
    task: TaskIdentifier,
    virtual_file_system: &'static VirtualFileSystem<'static>,
}

impl Drop for Standard {
    fn drop(&mut self) {
        let _ = block_on(self.virtual_file_system.close(self.standard_in, self.task));

        let _ = block_on(self.virtual_file_system.close(self.standard_out, self.task));

        let _ = block_on(
            self.virtual_file_system
                .close(self.standard_error, self.task),
        );
    }
}

impl Standard {
    pub async fn open(
        standard_in: &impl AsRef<Path>,
        standard_out: &impl AsRef<Path>,
        standard_error: &impl AsRef<Path>,
        task: TaskIdentifier,
        virtual_file_system: &'static VirtualFileSystem<'static>,
    ) -> Result<Self> {
        let standard_in = virtual_file_system
            .open(standard_in, Mode::READ_ONLY.into(), task)
            .await?;

        let standard_out = virtual_file_system
            .open(standard_out, Mode::WRITE_ONLY.into(), task)
            .await?;

        let standard_error = virtual_file_system
            .open(standard_error, Mode::WRITE_ONLY.into(), task)
            .await?;

        Ok(Self::new(
            standard_in,
            standard_out,
            standard_error,
            task,
            virtual_file_system,
        ))
    }

    pub fn new(
        standard_in: UniqueFileIdentifier,
        standard_out: UniqueFileIdentifier,
        standard_error: UniqueFileIdentifier,
        task: TaskIdentifier,
        virtual_file_system: &'static VirtualFileSystem,
    ) -> Self {
        Self {
            standard_in,
            standard_out,
            standard_error,
            task,
            virtual_file_system,
        }
    }

    pub async fn print(&self, arguments: &str) {
        let _ = self
            .virtual_file_system
            .write(self.standard_out, arguments.as_bytes(), self.task)
            .await;
    }

    pub async fn out_flush(&self) {
        self.virtual_file_system
            .flush(self.standard_out, self.task)
            .await
            .unwrap();
    }

    pub async fn write(&self, data: &[u8]) -> Size {
        match self
            .virtual_file_system
            .write(self.standard_out, data, self.task)
            .await
        {
            Ok(size) => size,
            Err(_) => 0_usize.into(),
        }
    }

    pub async fn print_line(&self, arguments: &str) {
        self.print(arguments).await;
        self.print("\n").await;
    }

    pub async fn print_error(&self, arguments: &str) {
        let _ = self
            .virtual_file_system
            .write(self.standard_error, arguments.as_bytes(), self.task)
            .await;
    }

    pub async fn print_error_line(&self, arguments: &str) {
        self.print_error(arguments).await;
        self.print_error("\n").await;
    }

    pub async fn read_line(&self, buffer: &mut String) {
        buffer.clear();

        let _ = self
            .virtual_file_system
            .read_line(self.standard_in, self.task, buffer)
            .await;
    }

    pub fn get_task(&self) -> TaskIdentifier {
        self.task
    }

    pub async fn duplicate(&self) -> Result<Self> {
        let standard_in = self
            .virtual_file_system
            .duplicate_file_identifier(self.standard_in, self.task)
            .await?;

        let standard_out = self
            .virtual_file_system
            .duplicate_file_identifier(self.standard_out, self.task)
            .await?;

        let standard_error = self
            .virtual_file_system
            .duplicate_file_identifier(self.standard_error, self.task)
            .await?;

        Ok(Self::new(
            standard_in,
            standard_out,
            standard_error,
            self.task,
            self.virtual_file_system,
        ))
    }

    pub fn split(
        &self,
    ) -> (
        UniqueFileIdentifier,
        UniqueFileIdentifier,
        UniqueFileIdentifier,
    ) {
        (self.standard_in, self.standard_out, self.standard_error)
    }

    pub(crate) async fn transfert(mut self, task: TaskIdentifier) -> Result<Self> {
        self.standard_in = self
            .virtual_file_system
            .transfert(
                self.standard_in,
                self.task,
                task,
                Some(FileIdentifier::STANDARD_IN),
            )
            .await?;

        self.standard_out = self
            .virtual_file_system
            .transfert(
                self.standard_out,
                self.task,
                task,
                Some(FileIdentifier::STANDARD_OUT),
            )
            .await?;

        self.standard_error = self
            .virtual_file_system
            .transfert(
                self.standard_error,
                self.task,
                task,
                Some(FileIdentifier::STANDARD_ERROR),
            )
            .await?;

        self.task = task;

        Ok(self)
    }
}
