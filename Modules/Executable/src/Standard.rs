use File_system::{File_identifier_type, Size_type, Unique_file_identifier_type};
use Task::Task_identifier_type;
use Virtual_file_system::Virtual_file_system_type;

use crate::Result_type;

pub struct Standard_type {
    Standard_in: Unique_file_identifier_type,
    Standard_out: Unique_file_identifier_type,
    Standard_error: Unique_file_identifier_type,
    Task: Task_identifier_type,
    Virtual_file_system: &'static Virtual_file_system_type,
}

impl Drop for Standard_type {
    fn drop(&mut self) {
        let _ = self.Virtual_file_system.Close(self.Standard_in, self.Task);

        let _ = self.Virtual_file_system.Close(self.Standard_out, self.Task);

        let _ = self
            .Virtual_file_system
            .Close(self.Standard_error, self.Task);
    }
}

impl Standard_type {
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

    pub fn Print(&self, Arguments: &str) {
        let _ = self
            .Virtual_file_system
            .Write(self.Standard_out, Arguments.as_bytes(), self.Task);
    }

    pub fn Out_flush(&self) {
        self.Virtual_file_system
            .Flush(self.Standard_out, self.Task)
            .unwrap();
    }

    pub fn Write(&self, Data: &[u8]) -> Size_type {
        match self
            .Virtual_file_system
            .Write(self.Standard_out, Data, self.Task)
        {
            Ok(Size) => Size,
            Err(_) => 0_usize.into(),
        }
    }

    pub fn Print_line(&self, Arguments: &str) {
        self.Print(Arguments);
        self.Print("\n");
    }

    pub fn Print_error(&self, Arguments: &str) {
        let _ =
            self.Virtual_file_system
                .Write(self.Standard_error, Arguments.as_bytes(), self.Task);
    }

    pub fn Print_error_line(&self, Arguments: &str) {
        self.Print_error(Arguments);
        self.Print_error("\n");
    }

    pub fn Read_line(&self, Buffer: &mut String) {
        Buffer.clear();

        let mut Current_buffer = [0u8; 1];

        loop {
            let Read = self
                .Virtual_file_system
                .Read(self.Standard_in, &mut Current_buffer, self.Task)
                .unwrap();

            if (Read != 1) || (Current_buffer[0] == b'\n') {
                break;
            }

            Buffer.push(Current_buffer[0] as char);
        }
    }

    pub fn Get_task(&self) -> Task_identifier_type {
        self.Task
    }

    pub fn Duplicate(&self) -> Result_type<Self> {
        let Standard_in = self
            .Virtual_file_system
            .Duplicate_file_identifier(self.Standard_in, self.Task)?;

        let Standard_out = self
            .Virtual_file_system
            .Duplicate_file_identifier(self.Standard_out, self.Task)?;

        let Standard_error = self
            .Virtual_file_system
            .Duplicate_file_identifier(self.Standard_error, self.Task)?;

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

    pub(crate) fn Transfert(mut self, Task: Task_identifier_type) -> Result_type<Self> {
        self.Standard_in = self.Virtual_file_system.Transfert(
            self.Standard_in,
            self.Task,
            Task,
            Some(File_identifier_type::Standard_in),
        )?;

        self.Standard_out = self.Virtual_file_system.Transfert(
            self.Standard_out,
            self.Task,
            Task,
            Some(File_identifier_type::Standard_out),
        )?;

        self.Standard_error = self.Virtual_file_system.Transfert(
            self.Standard_error,
            self.Task,
            Task,
            Some(File_identifier_type::Standard_error),
        )?;

        self.Task = Task;

        Ok(self)
    }
}
