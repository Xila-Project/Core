use alloc::string::String;
use File_system::{
    File_identifier_type, Mode_type, Path_type, Size_type, Unique_file_identifier_type,
};
use Futures::block_on;
use Task::Task_identifier_type;
use Virtual_file_system::Virtual_file_system_type;

use crate::Result_type;

pub struct Standard_type {
    standard_in: Unique_file_identifier_type,
    standard_out: Unique_file_identifier_type,
    standard_error: Unique_file_identifier_type,
    task: Task_identifier_type,
    virtual_file_system: &'static Virtual_file_system_type<'static>,
}

impl Drop for Standard_type {
    fn drop(&mut self) {
        let _ = block_on(self.virtual_file_system.Close(self.standard_in, self.task));

        let _ = block_on(self.virtual_file_system.Close(self.standard_out, self.task));

        let _ = block_on(
            self.virtual_file_system
                .Close(self.standard_error, self.task),
        );
    }
}

impl Standard_type {
    pub async fn open(
        standard_in: &impl AsRef<Path_type>,
        standard_out: &impl AsRef<Path_type>,
        standard_error: &impl AsRef<Path_type>,
        task: Task_identifier_type,
        virtual_file_system: &'static Virtual_file_system_type<'static>,
    ) -> Result_type<Self> {
        let standard_in = virtual_file_system
            .Open(standard_in, Mode_type::READ_ONLY.into(), task)
            .await?;

        let Standard_out = virtual_file_system
            .Open(standard_out, Mode_type::WRITE_ONLY.into(), task)
            .await?;

        let Standard_error = virtual_file_system
            .Open(standard_error, Mode_type::WRITE_ONLY.into(), task)
            .await?;

        Ok(Self::New(
            standard_in,
            Standard_out,
            Standard_error,
            task,
            virtual_file_system,
        ))
    }

    pub fn New(
        standard_in: Unique_file_identifier_type,
        standard_out: Unique_file_identifier_type,
        standard_error: Unique_file_identifier_type,
        task: Task_identifier_type,
        virtual_file_system: &'static Virtual_file_system_type,
    ) -> Self {
        Self {
            standard_in,
            standard_out,
            standard_error,
            task,
            virtual_file_system,
        }
    }

    pub async fn Print(&self, Arguments: &str) {
        let _ = self
            .virtual_file_system
            .Write(self.standard_out, Arguments.as_bytes(), self.task)
            .await;
    }

    pub async fn Out_flush(&self) {
        self.virtual_file_system
            .Flush(self.standard_out, self.task)
            .await
            .unwrap();
    }

    pub async fn Write(&self, Data: &[u8]) -> Size_type {
        match self
            .virtual_file_system
            .Write(self.standard_out, Data, self.task)
            .await
        {
            Ok(Size) => Size,
            Err(_) => 0_usize.into(),
        }
    }

    pub async fn Print_line(&self, Arguments: &str) {
        self.Print(Arguments).await;
        self.Print("\n").await;
    }

    pub async fn Print_error(&self, Arguments: &str) {
        let _ = self
            .virtual_file_system
            .Write(self.standard_error, Arguments.as_bytes(), self.task)
            .await;
    }

    pub async fn Print_error_line(&self, Arguments: &str) {
        self.Print_error(Arguments).await;
        self.Print_error("\n").await;
    }

    pub async fn Read_line(&self, Buffer: &mut String) {
        Buffer.clear();

        let _ = self
            .virtual_file_system
            .Read_line(self.standard_in, self.task, Buffer)
            .await;
    }

    pub fn Get_task(&self) -> Task_identifier_type {
        self.task
    }

    pub async fn Duplicate(&self) -> Result_type<Self> {
        let standard_in = self
            .virtual_file_system
            .Duplicate_file_identifier(self.standard_in, self.task)
            .await?;

        let Standard_out = self
            .virtual_file_system
            .Duplicate_file_identifier(self.standard_out, self.task)
            .await?;

        let Standard_error = self
            .virtual_file_system
            .Duplicate_file_identifier(self.standard_error, self.task)
            .await?;

        Ok(Self::New(
            standard_in,
            Standard_out,
            Standard_error,
            self.task,
            self.virtual_file_system,
        ))
    }

    pub fn Split(
        &self,
    ) -> (
        Unique_file_identifier_type,
        Unique_file_identifier_type,
        Unique_file_identifier_type,
    ) {
        (self.standard_in, self.standard_out, self.standard_error)
    }

    pub(crate) async fn Transfert(mut self, Task: Task_identifier_type) -> Result_type<Self> {
        self.standard_in = self
            .virtual_file_system
            .Transfert(
                self.standard_in,
                self.task,
                Task,
                Some(File_identifier_type::STANDARD_IN),
            )
            .await?;

        self.standard_out = self
            .virtual_file_system
            .Transfert(
                self.standard_out,
                self.task,
                Task,
                Some(File_identifier_type::STANDARD_OUT),
            )
            .await?;

        self.standard_error = self
            .virtual_file_system
            .Transfert(
                self.standard_error,
                self.task,
                Task,
                Some(File_identifier_type::STANDARD_ERROR),
            )
            .await?;

        self.task = Task;

        Ok(self)
    }
}
