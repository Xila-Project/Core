use std::{
    collections::{BTreeMap, HashMap},
    sync::RwLock,
    time::Duration,
};

use Task::{Task_identifier_type, Task_type};
use Users::{
    Group_identifier_type, Root_group_identifier, Root_user_identifier, User_identifier_type,
};

use crate::{
    Error_type, File_identifier_type, File_system_traits, Flags_type, Mode_type, Path_owned_type,
    Path_type, Permissions_type, Result_type, Size_type, Status_type, Type_type,
};

use super::Pipe_type;

struct Named_pipe_type {
    Pipe: Pipe_type,
    User: User_identifier_type,
    Group: User_identifier_type,
    Permissions: Permissions_type,
}

struct Inner_type {
    pub Named_pipes: HashMap<Path_owned_type, Named_pipe_type>,
    pub Opened_pipes: BTreeMap<u32, (Pipe_type, Flags_type)>,
}

pub struct File_system_type(RwLock<Inner_type>);

impl File_system_type {
    pub fn New() -> Self {
        Self(RwLock::new(Inner_type {
            Named_pipes: HashMap::new(),
            Opened_pipes: BTreeMap::new(),
        }))
    }

    /// Get the local file identifier from the task identifier and the file identifier.
    ///
    /// Since the file identifier must remain valid, this method is static, and the lock
    /// has to be acquired in the calling method.
    fn Get_new_file_identifier(
        Task: Task_identifier_type,
        Opened_pipes: &BTreeMap<u32, (Pipe_type, Flags_type)>,
    ) -> Result_type<File_identifier_type> {
        let Start = Self::Get_local_file_identifier(Task, File_identifier_type::from(0));
        let End = Self::Get_local_file_identifier(Task, File_identifier_type::from(0xFFFF));

        for File_identifier in Start..=End {
            if !Opened_pipes.contains_key(&File_identifier) {
                return Ok(File_identifier_type::from(File_identifier as u16));
                // Remove the task identifier and keep the file identifier.
            }
        }

        Err(Error_type::Too_many_open_files)
    }
}

impl File_system_traits for File_system_type {
    fn Create_named_pipe(&self, Path: &dyn AsRef<Path_type>, Size: Size_type) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        Inner.Named_pipes.insert(
            Path.as_ref().to_owned(),
            Named_pipe_type {
                Pipe: Pipe_type::New(Size.into()),
                User: Root_user_identifier,
                Group: Root_group_identifier,
                Permissions: Permissions_type::New_standard_file(),
            },
        );

        Ok(())
    }

    fn Create_unnamed_pipe(
        &self,
        Task_identifier: Task_identifier_type,
        Size: Size_type,
        Status: Status_type,
    ) -> Result_type<(File_identifier_type, File_identifier_type)> {
        let Pipe = Pipe_type::New(Size.into());

        let mut Inner = self.0.write()?;

        let File_identifier_read =
            Self::Get_new_file_identifier(Task_identifier, &Inner.Opened_pipes)?;

        Inner.Opened_pipes.insert(
            Self::Get_local_file_identifier(Task_identifier, File_identifier_read),
            (
                Pipe.clone(),
                Flags_type::New(Mode_type::Read_only(), Some(Status)),
            ),
        );

        let File_identifier_write =
            Self::Get_new_file_identifier(Task_identifier, &Inner.Opened_pipes)?;

        Inner.Opened_pipes.insert(
            Self::Get_local_file_identifier(Task_identifier, File_identifier_write),
            (Pipe, Flags_type::New(Mode_type::Write_only(), Some(Status))),
        );

        Ok((File_identifier_read, File_identifier_write))
    }

    fn Exists(&self, Path: &dyn AsRef<Path_type>) -> Result_type<bool> {
        Ok(self.0.read()?.Named_pipes.contains_key(Path.as_ref()))
    }

    fn Create_file(&self, _: &dyn AsRef<Path_type>) -> Result_type<()> {
        Result_type::Err(Error_type::Unsupported_operation)
    }

    fn Open(
        &self,
        Task: Task_identifier_type,
        Path: &dyn AsRef<Path_type>,
        Flags: Flags_type,
    ) -> Result_type<File_identifier_type> {
        let mut Inner = self.0.write()?;

        let Named_pipe = Inner
            .Named_pipes
            .get(Path.as_ref())
            .ok_or(Error_type::Not_found)?
            .Pipe
            .clone();

        let File_identifier = Self::Get_new_file_identifier(Task, &Inner.Opened_pipes)?;

        let Local_file_identifier = Self::Get_local_file_identifier(Task, File_identifier);

        Inner
            .Opened_pipes
            .insert(Local_file_identifier, (Named_pipe, Flags));

        Ok(File_identifier)
    }

    fn Close(&self, Task: Task_identifier_type, File: File_identifier_type) -> Result_type<()> {
        self.0
            .write()?
            .Opened_pipes
            .remove(&Self::Get_local_file_identifier(Task, File))
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(())
    }

    fn Close_all(&self, Task: Task_identifier_type) -> Result_type<()> {
        self.0
            .write()?
            .Opened_pipes
            .retain(|Key, _| Self::Decompose_local_file_identifier(*Key).0 != Task);

        Ok(())
    }

    fn Transfert_file_identifier(
        &self,
        Old_task: Task_identifier_type,
        New_task: Task_identifier_type,
        File: File_identifier_type,
    ) -> Result_type<File_identifier_type> {
        let Local_file_identifier = Self::Get_local_file_identifier(Old_task, File);

        let mut Inner = self.0.write()?;

        let New_file_identifier = Self::Get_new_file_identifier(New_task, &Inner.Opened_pipes)?;

        let (Pipe, Mode) = Inner
            .Opened_pipes
            .remove(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        let Local_file_identifier = Self::Get_local_file_identifier(New_task, New_file_identifier);

        Inner
            .Opened_pipes
            .insert(Local_file_identifier, (Pipe, Mode));

        Ok(New_file_identifier)
    }

    fn Delete(&self, Path: &dyn AsRef<Path_type>) -> Result_type<()> {
        self.0
            .write()?
            .Named_pipes
            .remove(Path.as_ref())
            .ok_or(Error_type::Not_found)?;

        Ok(())
    }

    fn Read(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        Buffer: &mut [u8],
    ) -> Result_type<Size_type> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);

        let Inner = self.0.read()?;

        let (Pipe, Flags) = Inner
            .Opened_pipes
            .get(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.Get_mode().Get_read() {
            return Err(Error_type::Invalid_mode);
        }

        while let Err(Error) = Pipe.Read(Buffer) {
            if let Error_type::Ressource_busy = Error {
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

    fn Write(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        Buffer: &[u8],
    ) -> Result_type<Size_type> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);

        let Inner = self.0.read()?;

        let (Pipe, Mode) = Inner
            .Opened_pipes
            .get(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Mode.Get_mode().Get_write() {
            return Err(Error_type::Invalid_mode);
        }

        while let Err(Error) = Pipe.Write(Buffer) {
            if let Error_type::Ressource_busy = Error {
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

    fn Move(
        &self,
        Source: &dyn AsRef<Path_type>,
        Destination: &dyn AsRef<Path_type>,
    ) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        let Pipe = Inner
            .Named_pipes
            .remove(Source.as_ref())
            .ok_or(Error_type::Not_found)?;

        Inner
            .Named_pipes
            .insert(Destination.as_ref().to_owned(), Pipe);

        Ok(())
    }

    fn Set_position(
        &self,
        _: Task_identifier_type,
        _: File_identifier_type,
        _: &crate::Position_type,
    ) -> Result_type<Size_type> {
        Err(Error_type::Unsupported_operation)
    }

    fn Flush(&self, _: Task_identifier_type, _: File_identifier_type) -> Result_type<()> {
        Ok(())
    }

    fn Get_type(&self, _: &dyn AsRef<Path_type>) -> Result_type<crate::Type_type> {
        Ok(Type_type::Named_pipe)
    }

    fn Get_size(&self, Path: &dyn AsRef<Path_type>) -> Result_type<Size_type> {
        self.0
            .read()?
            .Named_pipes
            .get(Path.as_ref())
            .ok_or(Error_type::Not_found)?
            .Pipe
            .Get_size()
    }

    fn Create_directory(&self, _: &dyn AsRef<Path_type>) -> Result_type<()> {
        Err(Error_type::Unsupported_operation)
    }

    fn Set_permissions(
        &self,
        Path: &dyn AsRef<Path_type>,
        Permissions_type: Permissions_type,
    ) -> Result_type<()> {
        self.0
            .write()?
            .Named_pipes
            .get_mut(Path.as_ref())
            .ok_or(Error_type::Not_found)?
            .Permissions = Permissions_type;

        Ok(())
    }

    fn Get_permissions(&self, Path: &dyn AsRef<Path_type>) -> Result_type<Permissions_type> {
        Ok(self
            .0
            .read()?
            .Named_pipes
            .get(Path.as_ref())
            .ok_or(Error_type::Not_found)?
            .Permissions)
    }

    fn Get_owner(
        &self,
        Path: &dyn AsRef<Path_type>,
    ) -> Result_type<(User_identifier_type, Group_identifier_type)> {
        self.0
            .read()?
            .Named_pipes
            .get(Path.as_ref())
            .ok_or(Error_type::Not_found)
            .map(|Named_pipe| (Named_pipe.User, Named_pipe.Group))
    }

    fn Set_owner(
        &self,
        Path: &dyn AsRef<Path_type>,
        User: Option<User_identifier_type>,
        Group: Option<Group_identifier_type>,
    ) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        let Named_pipe = Inner
            .Named_pipes
            .get_mut(Path.as_ref())
            .ok_or(Error_type::Not_found)?;

        if let Some(User) = User {
            Named_pipe.User = User;
        }

        if let Some(Group) = Group {
            Named_pipe.Group = Group;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Test_new() {
        let File_system = File_system_type::New();
        assert!(File_system.0.read().unwrap().Named_pipes.is_empty());
        assert!(File_system.0.read().unwrap().Opened_pipes.is_empty());
    }

    #[test]
    fn Test_get_local_file_identifier() {
        let Task_identifier = Task_identifier_type::from(1);
        let File_identifier = File_identifier_type::from(0);
        let local_file_id =
            File_system_type::Get_local_file_identifier(Task_identifier, File_identifier);
        assert_eq!(local_file_id, 65536); // 1 << 16 | 0
    }

    #[test]
    fn Test_new_unnamed_pipe() {
        let File_system = File_system_type::New();
        let Task_identifier = Task_identifier_type::from(1);
        let Size = 1024_usize;
        let (Read_identifier, Write_identifier) = File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Size.into(),
                Status_type::default().Set_non_blocking(true),
            )
            .unwrap();
        assert_ne!(Read_identifier, Write_identifier);
        assert!(File_system.0.read().unwrap().Opened_pipes.contains_key(
            &File_system_type::Get_local_file_identifier(Task_identifier, Read_identifier)
        ));
        assert!(File_system.0.read().unwrap().Opened_pipes.contains_key(
            &File_system_type::Get_local_file_identifier(Task_identifier, Write_identifier)
        ));
    }

    #[test]
    fn Test_close_all() {
        let File_system = File_system_type::New();
        let Task_identifier = Task_identifier_type::from(1);
        let Size = 1024_usize;
        File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Size.into(),
                Status_type::default().Set_non_blocking(true),
            )
            .unwrap();
        File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Size.into(),
                Status_type::default().Set_non_blocking(true),
            )
            .unwrap();
        File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Size.into(),
                Status_type::default().Set_non_blocking(true),
            )
            .unwrap();
        File_system.Close_all(Task_identifier).unwrap();
        assert!(File_system.0.read().unwrap().Opened_pipes.is_empty());
    }

    #[test]
    fn Test_delete_named_pipe() {
        let File_system = File_system_type::New();

        let Size = 1024_usize;
        let Path = Path_type::New("/named_pipe").unwrap();

        File_system.Create_named_pipe(&Path, Size.into()).unwrap();
        assert!(File_system.Delete(&Path).is_ok());
        assert!(File_system.0.read().unwrap().Named_pipes.is_empty());
        assert!(File_system.Delete(&Path).is_err());
    }

    #[test]
    fn Test_read_write_unnamed_pipe() {
        let File_system = File_system_type::New();
        let Task_identifier = Task_identifier_type::from(1);
        let Size = 1024_usize;
        let (Read_identifier, Write_identifier) = File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Size.into(),
                Status_type::default().Set_non_blocking(true),
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
