use alloc::{fmt, string::String};
use core::{fmt::Debug, mem::forget};
use file_system::{Mode, Path, UniqueFileIdentifier};
use task::TaskIdentifier;
use virtual_file_system::{File, VirtualFileSystem};

use crate::Result;

pub struct Standard {
    pub standard_in: File<'static>,
    pub standard_out: File<'static>,
    pub standard_error: File<'static>,
}

impl Debug for Standard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Standard")
            .field("standard_in", &self.standard_in)
            .field("standard_out", &self.standard_out)
            .field("standard_error", &self.standard_error)
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
        }
    }

    pub fn out(&mut self) -> &mut File<'static> {
        &mut self.standard_out
    }

    pub fn error(&mut self) -> &mut File<'static> {
        &mut self.standard_error
    }

    pub async fn read_line(&self, buffer: &mut String) {
        buffer.clear();

        let _ = self.standard_in.read_line(buffer).await;
    }

    pub fn get_task(&self) -> TaskIdentifier {
        self.standard_in.get_task()
    }

    pub async fn duplicate(&self) -> file_system::Result<Self> {
        Ok(Self {
            standard_in: self.standard_in.duplicate().await?,
            standard_out: self.standard_out.duplicate().await?,
            standard_error: self.standard_error.duplicate().await?,
        })
    }

    pub fn split(self) -> (File<'static>, File<'static>, File<'static>) {
        (self.standard_in, self.standard_out, self.standard_error)
    }

    pub fn into_file_identifiers(
        self,
    ) -> (
        UniqueFileIdentifier,
        UniqueFileIdentifier,
        UniqueFileIdentifier,
    ) {
        let result = (
            self.standard_in.get_file_identifier(),
            self.standard_out.get_file_identifier(),
            self.standard_error.get_file_identifier(),
        );

        forget(self); // Prevent Drop from being called

        result
    }

    pub async fn transfer(self, task: TaskIdentifier) -> file_system::Result<Self> {
        let standard_in = self.standard_in.transfer(task, None).await?;
        let standard_out = self.standard_out.transfer(task, None).await?;
        let standard_error = self.standard_error.transfer(task, None).await?;

        Ok(Self {
            standard_in,
            standard_out,
            standard_error,
        })
    }
}
