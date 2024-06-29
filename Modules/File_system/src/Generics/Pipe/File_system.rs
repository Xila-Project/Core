use std::{collections::HashMap, time::Duration};

use Task::{Task_identifier_type, Task_type};

use crate::Prelude::{
    Error_type, File_identifier_type, Flags_type, Mode_type, Result, Size_type, Status_type,
};

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct Named_pipe_identifier_type(u32);

impl Named_pipe_identifier_type {
    pub const MAX: u32 = 0xFFFF_FFFF;
}

impl From<u32> for Named_pipe_identifier_type {
    fn from(Identifier: u32) -> Self {
        Named_pipe_identifier_type(Identifier)
    }
}

impl From<Named_pipe_identifier_type> for u32 {
    fn from(Identifier: Named_pipe_identifier_type) -> Self {
        Identifier.0
    }
}

impl From<[u8; 4]> for Named_pipe_identifier_type {
    fn from(Identifier: [u8; 4]) -> Self {
        Named_pipe_identifier_type(u32::from_ne_bytes(Identifier))
    }
}

impl From<Named_pipe_identifier_type> for [u8; 4] {
    fn from(Identifier: Named_pipe_identifier_type) -> Self {
        Identifier.0.to_ne_bytes()
    }
}

use super::Pipe_type;

pub struct Pipes_file_system_type {
    Named_pipes: HashMap<Named_pipe_identifier_type, Pipe_type>,
    Opened_pipes: HashMap<u32, (Pipe_type, Flags_type)>,
}

impl Pipes_file_system_type {
    pub fn New() -> Self {
        Self {
            Named_pipes: HashMap::new(),
            Opened_pipes: HashMap::new(),
        }
    }

    fn Get_named_pipe_identifier(&self) -> Option<Named_pipe_identifier_type> {
        (0..=Named_pipe_identifier_type::MAX)
            .find(|&Identifier| !self.Named_pipes.contains_key(&Identifier.into()))
            .map(|Identifier| Identifier.into())
    }

    fn Get_local_file_identifier(
        Task_identifier: Task_identifier_type,
        File_identifier: File_identifier_type,
    ) -> u32 {
        let File_identifier: u16 = File_identifier.into();
        let Task_identifier: u32 = Task_identifier.into();
        Task_identifier << 16 | File_identifier as u32
    }

    fn Get_new_file_identifier(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Option<File_identifier_type> {
        let Start = Self::Get_local_file_identifier(Task_identifier, File_identifier_type::from(0));
        let End =
            Self::Get_local_file_identifier(Task_identifier, File_identifier_type::from(0xFFFF));

        for File_identifier in Start..=End {
            if !self.Opened_pipes.contains_key(&File_identifier) {
                return Some(File_identifier_type::from(File_identifier as u16));
                // Remove the task identifier and keep the file identifier.
            }
        }

        None
    }

    pub fn Create_named_pipe(&mut self, Size: usize) -> Result<Named_pipe_identifier_type> {
        let Identifier = self
            .Get_named_pipe_identifier()
            .ok_or(Error_type::Too_many_open_files)?;

        self.Named_pipes.insert(Identifier, Pipe_type::New(Size));

        Ok(Identifier)
    }

    pub fn Create_unnamed_pipe(
        &mut self,
        Task_identifier: Task_identifier_type,
        Status: Status_type,
        Size: usize,
    ) -> Result<(File_identifier_type, File_identifier_type)> {
        let Pipe = Pipe_type::New(Size);

        let New_file_identifier_read = self
            .Get_new_file_identifier(Task_identifier)
            .ok_or(Error_type::Too_many_open_files)?;

        self.Opened_pipes.insert(
            Self::Get_local_file_identifier(Task_identifier, New_file_identifier_read),
            (
                Pipe.clone(),
                Flags_type::New(Mode_type::Read_only(), Some(Status)),
            ),
        );

        let New_file_identifier_write = self
            .Get_new_file_identifier(Task_identifier)
            .ok_or(Error_type::Too_many_open_files)?;

        self.Opened_pipes.insert(
            Self::Get_local_file_identifier(Task_identifier, New_file_identifier_write),
            (Pipe, Flags_type::New(Mode_type::Write_only(), Some(Status))),
        );

        Ok((New_file_identifier_read, New_file_identifier_write))
    }

    pub fn Open(
        &mut self,
        Task_identifier: Task_identifier_type,
        Identifier: Named_pipe_identifier_type,
        Flags: Flags_type,
    ) -> Result<File_identifier_type> {
        let Named_pipe = self
            .Named_pipes
            .get(&Identifier)
            .ok_or(Error_type::Not_found)?;

        let File_identifier = self
            .Get_new_file_identifier(Task_identifier)
            .ok_or(Error_type::Too_many_open_files)?;

        let Local_file_identifier =
            Self::Get_local_file_identifier(Task_identifier, File_identifier);

        self.Opened_pipes
            .insert(Local_file_identifier, (Named_pipe.clone(), Flags));

        Ok(File_identifier)
    }

    pub fn Close(
        &mut self,
        Task: Task::Task_identifier_type,
        File: File_identifier_type,
    ) -> Result<()> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);

        self.Opened_pipes
            .remove(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(())
    }

    pub fn Close_all(&mut self, Task: Task::Task_identifier_type) -> Result<()> {
        let Start = Self::Get_local_file_identifier(Task, File_identifier_type::from(0));
        let End = Self::Get_local_file_identifier(Task, File_identifier_type::from(0xFFFF));

        self.Opened_pipes.retain(|Key, _| {
            if *Key >= Start && *Key <= End {
                return false;
            }
            true
        });

        Ok(())
    }

    pub fn Delete(&mut self, Identifier: Named_pipe_identifier_type) -> Result<()> {
        self.Named_pipes
            .remove(&Identifier)
            .ok_or(Error_type::Not_found)?;

        // The pipe is still opened by some tasks until they close it.
        // The pipe will be deleted when the last task closes it.

        Ok(())
    }

    pub fn Read(
        &mut self,
        Task: Task::Task_identifier_type,
        File: File_identifier_type,
        Buffer: &mut [u8],
    ) -> Result<Size_type> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);

        let (Pipe, Flags) = self
            .Opened_pipes
            .get(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.Get_mode().Get_read() {
            return Err(Error_type::Invalid_mode);
        }

        while let Err(Error) = Pipe.Read(Buffer) {
            if let Error_type::File_system_full = Error {
                if Flags.Get_status().Get_non_blocking() {
                    return Err(Error_type::Ressource_busy);
                } else {
                    Task_type::Sleep(Duration::from_millis(5));
                }
            } else {
                return Err(Error);
            }
        }

        Ok(Buffer.len().into())
    }

    pub fn Write(
        &mut self,
        Task: Task::Task_identifier_type,
        File: File_identifier_type,
        Buffer: &[u8],
    ) -> Result<Size_type> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);

        let (Pipe, Mode) = self
            .Opened_pipes
            .get(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Mode.Get_mode().Get_write() {
            return Err(Error_type::Invalid_mode);
        }

        while let Err(Error) = Pipe.Write(Buffer) {
            if let Error_type::File_system_full = Error {
                if Mode.Get_status().Get_non_blocking() {
                    return Err(Error_type::Ressource_busy);
                } else {
                    Task_type::Sleep(Duration::from_millis(5));
                }
            } else {
                return Err(Error);
            }
        }

        Ok(Buffer.len().into())
    }

    pub fn Get_size(&self, Identifier: Named_pipe_identifier_type) -> Result<Size_type> {
        self.Named_pipes
            .get(&Identifier)
            .ok_or(Error_type::Not_found)?
            .Get_size()
    }

    pub fn Transfert_file_identifier(
        &mut self,
        Old_task: Task_identifier_type,
        New_task: Task_identifier_type,
        File: File_identifier_type,
    ) -> Result<File_identifier_type> {
        let Local_file_identifier = Self::Get_local_file_identifier(Old_task, File);

        let (Pipe, Mode) = self
            .Opened_pipes
            .remove(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        let New_file_identifier = self
            .Get_new_file_identifier(New_task)
            .ok_or(Error_type::Too_many_open_files)?;

        let Local_file_identifier = Self::Get_local_file_identifier(New_task, New_file_identifier);

        self.Opened_pipes
            .insert(Local_file_identifier, (Pipe, Mode));

        Ok(New_file_identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Test_new() {
        let File_system = Pipes_file_system_type::New();
        assert!(File_system.Named_pipes.is_empty());
        assert!(File_system.Opened_pipes.is_empty());
    }

    #[test]
    fn Test_get_local_file_identifier() {
        let Task_identifier = Task_identifier_type::from(1);
        let File_identifier = File_identifier_type::from(0);
        let local_file_id =
            Pipes_file_system_type::Get_local_file_identifier(Task_identifier, File_identifier);
        assert_eq!(local_file_id, 65536); // 1 << 16 | 0
    }

    #[test]
    fn Test_new_unnamed_pipe() {
        let mut File_system = Pipes_file_system_type::New();
        let Task_identifier = Task_identifier_type::from(1);
        let Size = 1024;
        let (Read_identifier, Write_identifier) = File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Status_type::default().Set_non_blocking(true),
                Size,
            )
            .unwrap();
        assert_ne!(Read_identifier, Write_identifier);
        assert!(File_system.Opened_pipes.contains_key(
            &Pipes_file_system_type::Get_local_file_identifier(Task_identifier, Read_identifier)
        ));
        assert!(File_system.Opened_pipes.contains_key(
            &Pipes_file_system_type::Get_local_file_identifier(Task_identifier, Write_identifier)
        ));
    }

    #[test]
    fn Test_close_all() {
        let mut File_system = Pipes_file_system_type::New();
        let Task_identifier = Task_identifier_type::from(1);
        let Size = 1024;
        File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Status_type::default().Set_non_blocking(true),
                Size,
            )
            .unwrap();
        File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Status_type::default().Set_non_blocking(true),
                Size,
            )
            .unwrap();
        File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Status_type::default().Set_non_blocking(true),
                Size,
            )
            .unwrap();
        File_system.Close_all(Task_identifier).unwrap();
        assert!(File_system.Opened_pipes.is_empty());
    }

    #[test]
    fn Test_delete_named_pipe() {
        let mut File_system = Pipes_file_system_type::New();

        let Size = 1024;

        let Identifier = File_system.Create_named_pipe(Size).unwrap();
        assert!(File_system.Delete(Identifier).is_ok());
        assert!(File_system.Named_pipes.is_empty());
        assert!(File_system.Delete(Identifier).is_err());
    }

    #[test]
    fn Test_read_write_unnamed_pipe() {
        let mut File_system = Pipes_file_system_type::New();
        let Task_identifier = Task_identifier_type::from(1);
        let Size = 1024;
        let (Read_identifier, Write_identifier) = File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Status_type::default().Set_non_blocking(true),
                Size,
            )
            .unwrap();

        let Write_data = b"Hello, pipe!";
        File_system
            .Write(Task_identifier, Write_identifier, Write_data)
            .unwrap();

        let mut Buffer = [0; 12];

        let Read_data = File_system
            .Read(Task_identifier, Read_identifier, &mut Buffer)
            .unwrap();
        assert_eq!(Write_data, &Buffer[..Read_data.into()]);
    }
}
