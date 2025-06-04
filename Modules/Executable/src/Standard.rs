use File_system::{
    File_identifier_type, Mode_type, Path_type, Size_type, Unique_file_identifier_type,
};
use Futures::block_on;
use Task::Task_identifier_type;
use Virtual_file_system::Virtual_file_system_type;

use crate::Result_type;

pub struct Standard_type {
    Standard_in: Unique_file_identifier_type,
    Standard_out: Unique_file_identifier_type,
    Standard_error: Unique_file_identifier_type,
    Task: Task_identifier_type,
    Virtual_file_system: &'static Virtual_file_system_type<'static>,
}

impl Drop for Standard_type {
    fn drop(&mut self) {
        let _ = block_on(self.Virtual_file_system.Close(self.Standard_in, self.Task));

        let _ = block_on(self.Virtual_file_system.Close(self.Standard_out, self.Task));

        let _ = block_on(
            self.Virtual_file_system
                .Close(self.Standard_error, self.Task),
        );
    }
}

impl Standard_type {
    pub async fn Open(
        Standard_in: &impl AsRef<Path_type>,
        Standard_out: &impl AsRef<Path_type>,
        Standard_error: &impl AsRef<Path_type>,
        Task: Task_identifier_type,
        Virtual_file_system: &'static Virtual_file_system_type<'static>,
    ) -> Result_type<Self> {
        let Standard_in = Virtual_file_system
            .Open(Standard_in, Mode_type::Read_only.into(), Task)
            .await?;

        let Standard_out = Virtual_file_system
            .Open(Standard_out, Mode_type::Write_only.into(), Task)
            .await?;

        let Standard_error = Virtual_file_system
            .Open(Standard_error, Mode_type::Write_only.into(), Task)
            .await?;

        Ok(Self::New(
            Standard_in,
            Standard_out,
            Standard_error,
            Task,
            Virtual_file_system,
        ))
    }

    pub fn New(
        Standard_in: Unique_file_identifier_type,
        Standard_out: Unique_file_identifier_type,
        Standard_error: Unique_file_identifier_type,
        Task: Task_identifier_type,
        Virtual_file_system: &'static Virtual_file_system_type,
    ) -> Self {
        Self {
            Standard_in,
            Standard_out,
            Standard_error,
            Task,
            Virtual_file_system,
        }
    }

    pub async fn Print(&self, Arguments: &str) {
        let _ = self
            .Virtual_file_system
            .Write(self.Standard_out, Arguments.as_bytes(), self.Task)
            .await;
    }

    pub async fn Out_flush(&self) {
        self.Virtual_file_system
            .Flush(self.Standard_out, self.Task)
            .await
            .unwrap();
    }

    pub async fn Write(&self, Data: &[u8]) -> Size_type {
        match self
            .Virtual_file_system
            .Write(self.Standard_out, Data, self.Task)
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
            .Virtual_file_system
            .Write(self.Standard_error, Arguments.as_bytes(), self.Task)
            .await;
    }

    pub async fn Print_error_line(&self, Arguments: &str) {
        self.Print_error(Arguments).await;
        self.Print_error("\n").await;
    }

    pub async fn Read_line(&self, Buffer: &mut String) {
        Buffer.clear();

        let _ = self
            .Virtual_file_system
            .Read_line(self.Standard_in, self.Task, Buffer)
            .await;
    }

    pub fn Get_task(&self) -> Task_identifier_type {
        self.Task
    }

    pub async fn Duplicate(&self) -> Result_type<Self> {
        let Standard_in = self
            .Virtual_file_system
            .Duplicate_file_identifier(self.Standard_in, self.Task)
            .await?;

        let Standard_out = self
            .Virtual_file_system
            .Duplicate_file_identifier(self.Standard_out, self.Task)
            .await?;

        let Standard_error = self
            .Virtual_file_system
            .Duplicate_file_identifier(self.Standard_error, self.Task)
            .await?;

        Ok(Self::New(
            Standard_in,
            Standard_out,
            Standard_error,
            self.Task,
            self.Virtual_file_system,
        ))
    }

    pub fn Split(
        &self,
    ) -> (
        Unique_file_identifier_type,
        Unique_file_identifier_type,
        Unique_file_identifier_type,
    ) {
        (self.Standard_in, self.Standard_out, self.Standard_error)
    }

    pub(crate) async fn Transfert(mut self, Task: Task_identifier_type) -> Result_type<Self> {
        self.Standard_in = self
            .Virtual_file_system
            .Transfert(
                self.Standard_in,
                self.Task,
                Task,
                Some(File_identifier_type::Standard_in),
            )
            .await?;

        self.Standard_out = self
            .Virtual_file_system
            .Transfert(
                self.Standard_out,
                self.Task,
                Task,
                Some(File_identifier_type::Standard_out),
            )
            .await?;

        self.Standard_error = self
            .Virtual_file_system
            .Transfert(
                self.Standard_error,
                self.Task,
                Task,
                Some(File_identifier_type::Standard_error),
            )
            .await?;

        self.Task = Task;

        Ok(self)
    }
}
