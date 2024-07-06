use std::{collections::BTreeMap, sync::RwLock};

use Task::Task_identifier_type;
use Users::{Group_identifier_type, Manager_type, User_identifier_type};

use super::{
    Error_type, File_system_identifier_type, File_system_traits, Flags_type, Mode_type,
    Path_owned_type, Path_type, Permissions_type,
    Pipe::{self, Named_pipe_identifier_type},
    Position_type, Result, Size_type, Status_type, Type_type, Unique_file_identifier_type,
};

struct Internal_file_system_type {
    pub Mount_point: Path_owned_type,
    pub Inner: Box<dyn File_system_traits>,
}

#[derive(Clone)]
pub struct Virtual_file_system_type {
    /// A reference to the task manager.
    Task_manager: Task::Manager_type,
    /// User manager.
    User_manager: Users::Manager_type,
    /// Mounted file systems.
    File_systems: RwLock<BTreeMap<File_system_identifier_type, Internal_file_system_type>>,
}

impl Virtual_file_system_type {
    const Pipe_file_system_identifier: File_system_identifier_type =
        File_system_identifier_type::New_from(0xFF);

    const Named_pipe_extension: &'static str = ".Xila_pipe";

    pub fn New(Task_manager: Task::Manager_type, User_manager: Users::Manager_type) -> Self {
        Virtual_file_system_type {
            Task_manager,
            User_manager,
            File_systems: Arc::new(RwLock::new(HashMap::new())),
            Pipes_file_system: Arc::new(RwLock::new(Pipe::Manager_type::New())),
        }
    }

    fn Get_new_file_system_identifier(&self) -> Option<File_system_identifier_type> {
        let File_systems = self.File_systems.read().ok()?;

        let mut File_system_identifier = File_system_identifier_type::New();

        while File_systems.contains_key(&File_system_identifier)
            || File_system_identifier == Self::Pipe_file_system_identifier
        {
            File_system_identifier += 1;
        }

        Some(File_system_identifier)
    }

    /// Mount a file system at a given mount point.
    pub fn Mount(
        &self,
        File_system: Box<dyn File_system_traits>,
        Mount_point: impl AsRef<Path_type>,
    ) -> Result_type<File_system_identifier_type> {
            return Err(Error_type::Already_exists);
        }

        let File_system_identifier = self
            .Get_new_file_system_identifier()
            .ok_or(Error_type::Too_many_mounted_file_systems)?;

        if !Mount_point.as_ref().Is_absolute() {
            return Err(Error_type::Invalid_path);
        }

        self.File_systems.write()?.insert(
            File_system_identifier,
            Internal_file_system_type {
                Mount_point: Mount_point.as_ref().to_owned(),
                Inner: File_system,
            },
        );

        Ok(File_system_identifier)
    }

    ) -> Result_type<Box<dyn File_system_traits>> {
            .write()?
            .remove(&File_system_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(())
    }

    fn Get_file_system<'b>(
        File_systems: &'b HashMap<File_system_identifier_type, Internal_file_system_type>,
        Path: &'b dyn AsRef<Path_type>,
    ) -> Result<(File_system_identifier_type, &'b Path_type)> {
        let mut Result_score = 0;
        let mut Result: Option<(File_system_identifier_type, &'b Path_type)> = None;

        for (File_system_identifier, File_system) in File_systems.iter() {
            if let Some(Relative_path) = Path.as_ref().Strip_prefix(&File_system.Mount_point) {
                let Score = Relative_path.Get_length();
                if Score > Result_score {
                    Result_score = Score;
                    Result = Some((*File_system_identifier, Relative_path));
                }
            }
        }

        Result.ok_or(Error_type::Invalid_path)
    }

    pub fn Open(
        &self,
        Path: impl AsRef<Path_type>,
        Flags: Flags_type,
    ) -> Result<Unique_file_identifier_type> {
        let File_systems = self.File_systems.read()?; // Get the file systems

        let (File_system_identifier, Relative_path) = Self::Get_file_system(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?;

        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        // - Check permissions

        self.Check_permissions(File_system, Task_identifier, Relative_path, Flags)?;

        // - Open file

        match File_system
            .Inner
            .Open(Task_identifier, &Relative_path, Flags)
        {
            Ok(File_identifier) => Ok(Unique_file_identifier_type::New(
                File_system_identifier,
                File_identifier,
            )),
            Err(Error_type::Invalid_path) => {
                if let Some(Self::Named_pipe_extension) = Relative_path.Get_extension() {
                    return Err(Error_type::Invalid_path);
                }

                self.Open_named_pipe(Relative_path, Flags)
            }

            Err(e) => Err(e),
        }
    }

    fn Check_permissions(
        &self,
        File_system: &Internal_file_system_type,
        Task_identifier: Task_identifier_type,
        Relative_path: impl AsRef<Path_type>,
        Flags: Flags_type,
    ) -> Result<()> {
        // - Check permissions

        let Permissions = File_system
            .Inner
            .Get_permissions(Task_identifier, &Relative_path)?;

        let (Owner_user, Owner_group) = File_system
            .Inner
            .Get_owner(Task_identifier, &Relative_path)?;

        let Task_user = self.Task_manager.Get_owner(Task_identifier)?;

        let Permission = if Task_user == Owner_user {
            Permissions.Get_user()
        } else if self.User_manager.Is_in_group(Task_user, Owner_group) {
            Permissions.Get_group()
        } else {
            Permissions.Get_others()
        };

        if !Flags.Is_permission_granted(&Permission) {
            return Err(Error_type::Permission_denied);
        }

        Ok(())
    }

    pub fn Close(&self, File: Unique_file_identifier_type) -> Result_type<()> {
        let (File_system_identifier, File_identifier) = File.Split();

        if File_system_identifier == Self::Pipe_file_system_identifier {
            return self.Pipes_file_system.write()?.Close(
                self.Task_manager.Get_current_task_identifier()?,
                File_identifier,
            );
        }

        let File_systems = self.File_systems.read()?; // Get the file systems

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_identifier)?; // Get the file system

        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        File_system.Inner.Close(Task_identifier, File_identifier)
    }

    pub fn Read(
        &self,
        File_identifier: Unique_file_identifier_type,
        Buffer: &mut [u8],
    ) -> Result_type<Size_type> {
        let (File_system_identifier, File_identifier) = File_identifier.Split();

        if File_system_identifier == Self::Pipe_file_system_identifier {
            return self.Pipes_file_system.write()?.Read(
                self.Task_manager.Get_current_task_identifier()?,
                File_identifier,
                Buffer,
            );
        }

        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        let File_systems = self.File_systems.read()?; // Get the file systems

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_identifier)?; // Get the file system

        File_system
            .Inner
            .Read(Task_identifier, File_identifier, Buffer)
    }

    pub fn Write(
        &self,
        File: Unique_file_identifier_type,
        Buffer: &[u8],
    ) -> Result_type<Size_type> {
        let (File_system_identifier, File_identifier) = File.Split();

        if File_system_identifier == Self::Pipe_file_system_identifier {
            return self.Pipes_file_system.write()?.Write(
                self.Task_manager.Get_current_task_identifier()?,
                File_identifier,
                Buffer,
            );
        }

        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        let File_systems = self.File_systems.read()?; // Get the file systems

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        File_system
            .Inner
            .Write(Task_identifier, File_identifier, Buffer)
    }

    pub fn Set_position(
        &self,
        File_identifier: Unique_file_identifier_type,
        Position: &Position_type,
    ) -> Result_type<Size_type> {
        let (File_system_identifier, File_identifier) = File_identifier.Split();

        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        let File_systems = self.File_systems.read()?; // Get the file systems

        let (File_system_identifier, File_identifier) = File_identifier.Split();

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_identifier)?; // Get the file system

        File_system
            .Inner
            .Set_position(Task_identifier, File_identifier, Position)
    }

    pub fn Exists(&self, Path: impl AsRef<Path_type>) -> Result<bool> {
        let File_systems = self.File_systems.read()?; // Get the file systems

        if File_systems.is_empty() {
            return Ok(false);
        }

        let (File_system_identifier, Relative_path) = Self::Get_file_system(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?; // Get the file system

        File_system.Inner.Exists(&Relative_path)
    }

    pub fn Get_size(&self, Path: impl AsRef<Path_type>) -> Result<Size_type> {
        let File_systems = self.File_systems.read()?; // Get the file system

        let (File_system_identifier, Relative_path) = Self::Get_file_system(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?; // Get the file system

        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        File_system.Inner.Get_size(Task_identifier, &Relative_path)
    }

    pub fn Get_type(&self, Path: impl AsRef<Path_type>) -> Result<Type_type> {
        let File_systems = self.File_systems.read()?; // Get the file systems

        let (File_system_identifier, Relative_path) = Self::Get_file_system(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?;

        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        File_system.Inner.Get_type(Task_identifier, &Relative_path)
    }

    pub fn Get_permissions(&self, Path: impl AsRef<Path_type>) -> Result<Permissions_type> {
        if Path.as_ref().Is_root() {
            return Ok(Permissions_type::New_all_full());
        }

        let File_systems = self.File_systems.read()?; // Get the file systems

        let (File_system_identifier, Relative_path) = Self::Get_file_system(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?;

        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        File_system
            .Inner
            .Get_permissions(Task_identifier, &Relative_path)
    }

    pub fn Get_owner(
        &self,
        Path: impl AsRef<Path_type>,
    ) -> Result<(User_identifier_type, Group_identifier_type)> {
        let File_systems = self.File_systems.read()?; // Get the file systems

        let (File_system_identifier, Relative_path) = Self::Get_file_system(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?;

        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        File_system.Inner.Get_owner(Task_identifier, &Relative_path)
    }

    pub fn Set_owner(
        &self,
        Path: impl AsRef<Path_type>,
        User: Option<User_identifier_type>,
        Group: Option<Group_identifier_type>,
    ) -> Result<()> {
        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        let Task_user = self.Task_manager.Get_owner(Task_identifier)?;

        if !Manager_type::Is_root(Task_user) {
            return Err(Error_type::Permission_denied);
        }

        let File_systems = self.File_systems.read()?;

        let (File_system_identifier, Relative_path) = Self::Get_file_system(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let File_systems = self.File_systems.read()?; // Get the file systems

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?;

        File_system
            .Inner
            .Set_owner(Task_identifier, &Relative_path, User, Group)
    }

    pub fn Set_permissions(
        &self,
        Path: impl AsRef<Path_type>,
        Permissions: &Permissions_type,
    ) -> Result<()> {
        if Path.as_ref().Is_root() {
            return Err(Error_type::Permission_denied);
        }

        let File_systems = self.File_systems.read()?;

        let (File_system_identifier, Relative_path) = Self::Get_file_system(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let File_systems = self.File_systems.read()?; // Get the file systems

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?;

        // - Check permissions

        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        let Task_user = self.Task_manager.Get_owner(Task_identifier)?;

        let Owner = if Manager_type::Is_root(Task_user) {
            true
        } else {
            let (Owner_user, _) = File_system
                .Inner
                .Get_owner(Task_identifier, &Relative_path)?;

            Task_user == Owner_user
        };

        if !Owner {
            return Err(Error_type::Permission_denied);
        }

        File_system
            .Inner
            .Set_permissions(Task_identifier, Permissions, &Relative_path)
    }

    pub fn Close_all(&self, Task_identifier: Task_identifier_type) -> Result<()> {
        let File_systems = self.File_systems.read()?; // Get the file systems

        for File_system in File_systems.values() {
            File_system.Inner.Close_all(Task_identifier)?;
        }

        self.Pipes_file_system.write()?.Close_all(Task_identifier)?;

        Ok(())
    }

    pub fn Create_named_pipe(&self, Path: &impl AsRef<Path_type>, Size: usize) -> Result<()> {
        let Path = Path
            .as_ref()
            .Set_extension(Self::Named_pipe_extension)
            .ok_or(Error_type::Invalid_path)?; // Append the pipe extension

        self.Create_file(&Path)?; // Create the special file

        let File = self.Open(&Path, Mode_type::Write_only().into())?; // Open the file

        let Pipe_identifier = self.Pipes_file_system.write()?.Create_named_pipe(Size)?; // Create the named pipe

        let Pipe_identifier: [u8; 4] = Pipe_identifier.into();

        self.Write(File, &Pipe_identifier)?; // Write the pipe identifier

        Ok(())
    }

    pub fn Create_unnamed_pipe(
        &self,
        Size: usize,
        Status: Status_type,
    ) -> Result<(Unique_file_identifier_type, Unique_file_identifier_type)> {
        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        let (File_identifier_read, File_identifier_write) = self
            .Pipes_file_system
            .write()?
            .Create_unnamed_pipe(Task_identifier, Status, Size)?; // Create the unnamed pipe

        Ok((
            Unique_file_identifier_type::New(
                Self::Pipe_file_system_identifier,
                File_identifier_read,
            ),
            Unique_file_identifier_type::New(
                Self::Pipe_file_system_identifier,
                File_identifier_write,
            ),
        ))
    }

    pub fn Create_file(&self, Path: impl AsRef<Path_type>) -> Result<()> {
        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        let File_systems = self.File_systems.read()?; // Get the file systems

        let (File_system_identifier, Relative_path) = Self::Get_file_system(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?;

        File_system
            .Inner
            .Create_file(Task_identifier, &Relative_path)
    }

    pub fn Create_directory(&self, Path: impl AsRef<Path_type>, Recursive: bool) -> Result<()> {
        if Recursive {
            // If the directory already exists, return Ok(()) (only if recursive is true).
            if self.Exists(Path.as_ref())? {
                return Ok(());
            }

            // Create the parent directory recursively.
            self.Create_directory(
                Path.as_ref().Go_parent().ok_or(Error_type::Invalid_path)?,
                true,
            )?
        }

        // Create current directory.
        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        let File_systems = self.File_systems.read()?; // Get the file systems

        let (File_system_identifier, Relative_path) = Self::Get_file_system(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?;

        File_system
            .Inner
            .Create_directory(Task_identifier, &Relative_path)
    }

    pub fn Delete(&self, Path: impl AsRef<Path_type>, Recursive: bool) -> Result_type<()> {
        if Recursive {
            todo!()
        }

        // Delete current directory.
        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        let File_systems = self.File_systems.read()?; // Get the file systems

        let (File_system_identifier, Relative_path) = Self::Get_file_system(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?;

        // Check if the user has the right to delete the file (write permission on the parent directory)
        self.Check_permissions(
            File_system,
            Task_identifier,
            Relative_path.Go_parent().ok_or(Error_type::Invalid_path)?,
            Mode_type::Write_only().into(),
        )?;

        File_system.Inner.Delete(&Relative_path)
    }

    pub fn Transfert_file(
        &self,
        File: Unique_file_identifier_type,
        New_task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        let File_systems = self.File_systems.read()?; // Get the file systems

        let (File_system_identifier, File_identifier) = File.Split();

        let New_file_identifier = if File_system_identifier == Self::Pipe_file_system_identifier {
            self.Pipes_file_system.write()?.Transfert_file_identifier(
                Task_identifier,
                New_task,
                File_identifier,
            )?
        } else {
            File_systems
                .get(&File_system_identifier)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Transfert_file_identifier(Task_identifier, New_task, File_identifier)?
        };

        Ok(Unique_file_identifier_type::New(
            File_system_identifier,
            New_file_identifier,
        ))
    }

    pub fn Flush(&self, File: Unique_file_identifier_type) -> Result_type<()> {
        let (File_system_identifier, File_identifier) = File.Split();

        if File_system_identifier == Self::Pipe_file_system_identifier {
            return Ok(()); // No need to flush a pipe.
                           // ? : Maybe we should return an error.
        }

        let Task_identifier = self.Task_manager.Get_current_task_identifier()?;

        let File_systems = self.File_systems.read()?; // Get the file systems

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_identifier)?; // Get the file system

        File_system.Inner.Flush(Task_identifier, File_identifier)
    }
}
