use alloc::{fmt, string::String};
use core::fmt::Debug;
use file_system::{FileIdentifier, Mode, Path, Size, UniqueFileIdentifier};
use task::TaskIdentifier;
use virtual_file_system::{File, VirtualFileSystem};

use crate::Result;

pub struct Standard {
    pub standard_in: File<'static>,
    pub standard_out: File<'static>,
    pub standard_error: File<'static>,
    pub task: TaskIdentifier,
}

impl Debug for Standard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Standard")
            .field("standard_in", &self.standard_in)
            .field("standard_out", &self.standard_out)
            .field("standard_error", &self.standard_error)
            .field("task", &self.task)
            .finish()
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
            standard_in: File::from(standard_in, virtual_file_system, task),
            standard_out: File::from(standard_out, virtual_file_system, task),
            standard_error: File::from(standard_error, virtual_file_system, task),
            task,
        }
    }

    pub async fn print(&self, arguments: &str) {
        let _ = self.standard_out.write(arguments.as_bytes()).await;
    }

    pub async fn out_flush(&self) {
        self.standard_out.flush().await.unwrap();
    }

    pub async fn write(&self, data: &[u8]) -> Size {
        match self.standard_out.write(data).await {
            Ok(size) => size,
            Err(_) => 0_usize.into(),
        }
    }

    pub async fn print_line(&self, arguments: &str) {
        self.print(arguments).await;
        self.print("\n").await;
    }

    pub async fn print_error(&self, arguments: &str) {
        let _ = self.standard_error.write(arguments.as_bytes()).await;
    }

    pub async fn print_error_line(&self, arguments: &str) {
        self.print_error(arguments).await;
        self.print_error("\n").await;
    }

    pub async fn read_line(&self, buffer: &mut String) {
        buffer.clear();

        let _ = self.standard_in.read_line(buffer).await;
    }

    pub fn get_task(&self) -> TaskIdentifier {
        self.task
    }

    pub async fn duplicate(&self) -> file_system::Result<Self> {
        Ok(Self {
            standard_in: self.standard_in.duplicate().await?,
            standard_out: self.standard_out.duplicate().await?,
            standard_error: self.standard_error.duplicate().await?,
            task: self.task,
        })
    }

    pub fn split(&self) -> (&File<'static>, &File<'static>, &File<'static>) {
        (&self.standard_in, &self.standard_out, &self.standard_error)
    }

    pub async fn transfer(mut self, task: TaskIdentifier) -> file_system::Result<Self> {
        self.standard_in = self
            .standard_in
            .transfer(task, Some(FileIdentifier::STANDARD_IN))
            .await?;
        self.standard_out = self
            .standard_out
            .transfer(task, Some(FileIdentifier::STANDARD_OUT))
            .await?;
        self.standard_error = self
            .standard_error
            .transfer(task, Some(FileIdentifier::STANDARD_ERROR))
            .await?;

        self.task = task;

        Ok(self)
    }
}
