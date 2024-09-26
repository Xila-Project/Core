use std::{
    collections::{BTreeMap, HashMap},
    sync::{
        atomic::{AtomicUsize, Ordering},
        RwLock,
    },
    time::Duration,
};

use Task::Task_identifier_type;
use Users::{Group_identifier_type, User_identifier_type};

use crate::{
    Error_type, File_identifier_inner_type, File_identifier_type, File_system_identifier_type,
    File_system_traits, Flags_type, Mode_type, Path_owned_type, Path_type, Permissions_type,
    Result_type, Size_type, Statistics_type, Status_type,
};

use super::Pipe_type;

enum Inner_item_type {
    Pipe(Pipe_type),
    Directory(AtomicUsize),
}

impl Clone for Inner_item_type {
    fn clone(&self) -> Self {
        match self {
            Self::Pipe(Pipe) => Self::Pipe(Pipe.clone()),
            Self::Directory(Counter) => {
                Self::Directory(AtomicUsize::new(Counter.load(Ordering::Acquire)))
            }
        }
    }
}

struct Inner_type {
    pub Named_pipes: HashMap<Path_owned_type, Pipe_type>,
    pub Opened_pipes: BTreeMap<usize, (Inner_item_type, Flags_type)>,
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
    fn Get_new_file_identifier<T>(
        Task: Task_identifier_type,
        Opened_pipes: &BTreeMap<usize, T>,
    ) -> Result_type<File_identifier_type> {
        let Start = Self::Get_local_file_identifier(Task, File_identifier_type::from(0));
        let End = Self::Get_local_file_identifier(Task, File_identifier_type::from(0xFFFF));

        for File_identifier in Start..=End {
            if !Opened_pipes.contains_key(&File_identifier) {
                return Ok(File_identifier_type::from(
                    File_identifier as File_identifier_inner_type,
                ));
                // Remove the task identifier and keep the file identifier.
            }
        }

        Err(Error_type::Too_many_open_files)
    }
}

impl File_system_traits for File_system_type {
    fn Create_named_pipe(
        &self,
        Path: &dyn AsRef<Path_type>,
        Size: Size_type,
        User: User_identifier_type,
        Group: Group_identifier_type,
        Permissions: Permissions_type,
    ) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        Inner.Named_pipes.insert(
            Path.as_ref().to_owned(),
            Pipe_type::New(Size.into(), User, Group, Permissions),
        );

        Ok(())
    }

    fn Create_unnamed_pipe(
        &self,
        Task_identifier: Task_identifier_type,
        Size: Size_type,
        Status: Status_type,
        User: User_identifier_type,
        Group: Group_identifier_type,
        Permissions: Permissions_type,
    ) -> Result_type<(File_identifier_type, File_identifier_type)> {
        let Pipe = Pipe_type::New(Size.into(), User, Group, Permissions);

        let mut Inner = self.0.write()?;

        let File_identifier_read =
            Self::Get_new_file_identifier(Task_identifier, &Inner.Opened_pipes)?;

        Inner.Opened_pipes.insert(
            Self::Get_local_file_identifier(Task_identifier, File_identifier_read),
            (
                Inner_item_type::Pipe(Pipe.clone()),
                Flags_type::New(Mode_type::Read_only, None, Some(Status)),
            ),
        );

        let File_identifier_write =
            Self::Get_new_file_identifier(Task_identifier, &Inner.Opened_pipes)?;

        Inner.Opened_pipes.insert(
            Self::Get_local_file_identifier(Task_identifier, File_identifier_write),
            (
                Inner_item_type::Pipe(Pipe),
                Flags_type::New(Mode_type::Write_only, None, Some(Status)),
            ),
        );

        Ok((File_identifier_read, File_identifier_write))
    }

    fn Exists(&self, Path: &dyn AsRef<Path_type>) -> Result_type<bool> {
        Ok(self.0.read()?.Named_pipes.contains_key(Path.as_ref()))
    }

    fn Open(
        &self,
        Task: Task_identifier_type,
        Path: &dyn AsRef<Path_type>,
        Flags: Flags_type,
    ) -> Result_type<File_identifier_type> {
        if Flags.Get_open().Get_create() || Flags.Get_open().Get_create_only() {
            return Err(Error_type::Invalid_flags);
        }

        let mut Inner = self.0.write()?;

        let Named_pipe = Inner
            .Named_pipes
            .get(Path.as_ref())
            .ok_or(Error_type::Not_found)?
            .clone();

        let File_identifier = Self::Get_new_file_identifier(Task, &Inner.Opened_pipes)?;

        let Local_file_identifier = Self::Get_local_file_identifier(Task, File_identifier);

        // - Open a directory.
        if Flags.Get_open().Get_directory() {
            let mut Found = false;
            for (Path, _) in Inner.Named_pipes.iter() {
                if Path.Go_parent().ok_or(Error_type::Internal_error)? == Path.as_ref() {
                    Found = true;
                    break;
                }
            }

            if !Found {
                return Err(Error_type::Not_found);
            }

            Inner.Opened_pipes.insert(
                Local_file_identifier,
                (Inner_item_type::Directory(AtomicUsize::new(0)), Flags),
            );
        }
        // - Open a file.
        else {
            Inner.Opened_pipes.insert(
                Local_file_identifier,
                (Inner_item_type::Pipe(Named_pipe), Flags),
            );
        }

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
        New_file_identifier: Option<File_identifier_type>,
    ) -> Result_type<File_identifier_type> {
        let Old_local_file_identifier = Self::Get_local_file_identifier(Old_task, File);

        let mut Inner = self.0.write()?;

        let New_file_identifier = if let Some(New_file_identifier) = New_file_identifier {
            New_file_identifier
        } else {
            Self::Get_new_file_identifier(New_task, &Inner.Opened_pipes)?
        };

        let New_local_file_identifier =
            Self::Get_local_file_identifier(New_task, New_file_identifier);

        if Inner.Opened_pipes.contains_key(&New_local_file_identifier) {
            return Err(Error_type::Invalid_identifier);
        }

        let (Pipe, Mode) = Inner
            .Opened_pipes
            .remove(&Old_local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        if Inner
            .Opened_pipes
            .insert(New_local_file_identifier, (Pipe, Mode))
            .is_some()
        {
            // Should never happen.
            return Err(Error_type::Invalid_identifier);
        }

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
                    Task::Manager_type::Sleep(Duration::from_millis(5));
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
                    Task::Manager_type::Sleep(Duration::from_millis(5));
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
            .Set_permissions(Permissions_type)?;

        Ok(())
    }

    fn Set_owner(
        &self,
        Path: &dyn AsRef<Path_type>,
        User: Option<User_identifier_type>,
        Group: Option<Group_identifier_type>,
    ) -> Result_type<()> {
        self.0
            .write()?
            .Named_pipes
            .get_mut(Path.as_ref())
            .ok_or(Error_type::Not_found)?
            .Set_owner(User, Group)
    }

    fn Get_statistics(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        File_system: File_system_identifier_type,
    ) -> Result_type<Statistics_type> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);

        let Inner = self.0.read()?;

        let (Pipe, _) = Inner
            .Opened_pipes
            .get(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        Pipe.Get_statistics(File_system, Local_file_identifier as u64)
    }

    fn Get_mode(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
    ) -> Result_type<Mode_type> {
        Ok(self
            .0
            .read()?
            .Opened_pipes
            .get(&Self::Get_local_file_identifier(Task, File))
            .ok_or(Error_type::Invalid_identifier)?
            .1
            .Get_mode())
    }

    fn Duplicate_file_identifier(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
    ) -> Result_type<File_identifier_type> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);

        let (Pipe, Mode) = self
            .0
            .write()?
            .Opened_pipes
            .get(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?
            .clone();

        let mut Inner = self.0.write()?;

        let New_file_identifier = Self::Get_new_file_identifier(Task, &Inner.Opened_pipes)?;

        let Local_file_identifier = Self::Get_local_file_identifier(Task, New_file_identifier);

        if Inner
            .Opened_pipes
            .insert(Local_file_identifier, (Pipe, Mode))
            .is_some()
        {
            // Should never happen.
            return Err(Error_type::Internal_error);
        }

        Ok(New_file_identifier)
    }
}

#[cfg(test)]
mod Tests {

    use Users::{Root_group_identifier, Root_user_identifier};

    use crate::Type_type;

    use super::*;

    const Permissions: Permissions_type = Permissions_type::New_default(Type_type::File);

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

        assert_eq!(local_file_id, 1 << (File_identifier_inner_type::BITS));
    }

    fn Initialize_file_system() -> File_system_type {
        if !Time::Is_initialized() {
            let Time_driver = Drivers::Native::Time_driver_type::New();

            Time::Initialize(Box::new(Time_driver)).expect("Error initializing time manager");
        }

        File_system_type::New()
    }

    #[test]
    fn Test_new_unnamed_pipe() {
        let File_system = Initialize_file_system();
        let Task_identifier = Task_identifier_type::from(1);
        let Size = 1024_usize;
        let (Read_identifier, Write_identifier) = File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Size.into(),
                Status_type::default().Set_non_blocking(true),
                Root_user_identifier,
                Root_group_identifier,
                Permissions,
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
        let File_system = Initialize_file_system();
        let Task_identifier = Task_identifier_type::from(1);
        let Size = 1024_usize;
        File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Size.into(),
                Status_type::default().Set_non_blocking(true),
                Root_user_identifier,
                Root_group_identifier,
                Permissions,
            )
            .unwrap();
        File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Size.into(),
                Status_type::default().Set_non_blocking(true),
                Root_user_identifier,
                Root_group_identifier,
                Permissions,
            )
            .unwrap();
        File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Size.into(),
                Status_type::default().Set_non_blocking(true),
                Root_user_identifier,
                Root_group_identifier,
                Permissions,
            )
            .unwrap();
        File_system.Close_all(Task_identifier).unwrap();
        assert!(File_system.0.read().unwrap().Opened_pipes.is_empty());
    }

    #[test]
    fn Test_delete_named_pipe() {
        let File_system = Initialize_file_system();
        let Size = 1024_usize;
        let Path = Path_type::New("/named_pipe").unwrap();

        File_system
            .Create_named_pipe(
                &Path,
                Size.into(),
                Root_user_identifier,
                Root_group_identifier,
                Permissions,
            )
            .unwrap();

        assert!(File_system.Delete(&Path).is_ok());
        assert!(File_system.0.read().unwrap().Named_pipes.is_empty());
        assert!(File_system.Delete(&Path).is_err());
    }

    #[test]
    fn Test_read_write_unnamed_pipe() {
        let File_system = Initialize_file_system();
        let Task_identifier = Task_identifier_type::from(1);
        let Size = 1024_usize;
        let (Read_identifier, Write_identifier) = File_system
            .Create_unnamed_pipe(
                Task_identifier,
                Size.into(),
                Status_type::default().Set_non_blocking(true),
                Root_user_identifier,
                Root_group_identifier,
                Permissions,
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
