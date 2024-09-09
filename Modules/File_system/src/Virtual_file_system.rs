use std::{collections::BTreeMap, sync::RwLock};

use Task::Task_identifier_type;
use Users::{Group_identifier_type, User_identifier_type};

use crate::{File_identifier_type, Mode_type, Statistics_type};

use super::{
    Device, Device_trait, Error_type, File_system_identifier_type, File_system_traits, Flags_type,
    Path_owned_type, Path_type, Permission_type, Permissions_type, Pipe, Position_type,
    Result_type, Size_type, Status_type, Type_type, Unique_file_identifier_type,
};

struct Internal_file_system_type {
    pub Mount_point: Option<Path_owned_type>,
    pub Inner: Box<dyn File_system_traits>,
}

/// Instance of the virtual file system.
///
/// # Safety
/// I know, it is not safe to use mutable static variables.
/// It is thread safe (after initialization) because it is only read after initialization.
/// It is a pragmatic choice for efficiency in embedded systems contexts (avoid using Arc).
static mut Virtual_file_system_instance: Option<Virtual_file_system_type> = None;

pub fn Initialize() -> Result_type<&'static Virtual_file_system_type> {
    unsafe {
        if Is_initialized() {
            return Err(Error_type::Already_initialized);
        }

        let Task_manager = Task::Get_instance();

        let User_manager =
            Users::Get_instance().map_err(|_| Error_type::Failed_to_get_users_manager_instance)?; // Get the user manager (it must be initialized before the file system

        Virtual_file_system_instance
            .replace(Virtual_file_system_type::New(Task_manager, User_manager)?);

        Ok(Get_instance())
    }
}

pub fn Is_initialized() -> bool {
    unsafe { Virtual_file_system_instance.is_some() }
}

pub fn Get_instance() -> &'static Virtual_file_system_type {
    unsafe {
        Virtual_file_system_instance
            .as_ref()
            .expect("File system not initialized")
    }
}

/// The virtual file system.
///
/// It is a singleton.
pub struct Virtual_file_system_type {
    /// A reference to the task manager.
    Task_manager: &'static Task::Manager_type,
    /// User manager.
    User_manager: &'static Users::Manager_type,
    /// Mounted file systems.
    File_systems: RwLock<BTreeMap<File_system_identifier_type, Internal_file_system_type>>,
}

impl Virtual_file_system_type {
    const Pipe_file_system_identifier: File_system_identifier_type =
        File_system_identifier_type::New(0);
    const Device_file_system_identifier: File_system_identifier_type =
        File_system_identifier_type::New(1);

    pub const Standard_input_file_identifier: File_identifier_type = File_identifier_type::New(0);
    pub const Standard_output_file_identifier: File_identifier_type = File_identifier_type::New(1);
    pub const Standard_error_file_identifier: File_identifier_type = File_identifier_type::New(2);

    fn New(
        Task_manager: &'static Task::Manager_type,
        User_manager: &'static Users::Manager_type,
    ) -> Result_type<Self> {
        let mut File_systems = BTreeMap::new();

        let Pipe_file_system = Pipe::File_system_type::New();

        File_systems.insert(
            Self::Pipe_file_system_identifier,
            Internal_file_system_type {
                Mount_point: None,
                Inner: Box::new(Pipe_file_system),
            },
        );

        let Device_file_system = Device::File_system_type::New();

        File_systems.insert(
            Self::Device_file_system_identifier,
            Internal_file_system_type {
                Mount_point: None,
                Inner: Box::new(Device_file_system),
            },
        );

        Ok(Self {
            Task_manager,
            User_manager,
            File_systems: RwLock::new(File_systems),
        })
    }

    fn Get_new_file_system_identifier(
        File_systems: &BTreeMap<File_system_identifier_type, Internal_file_system_type>,
    ) -> Option<File_system_identifier_type> {
        let mut File_system_identifier = File_system_identifier_type::New(0);

        while File_systems.contains_key(&File_system_identifier) {
            File_system_identifier += 1;
        }

        Some(File_system_identifier)
    }

    /// Try to execute a closure on the concerned file systems on an **existing** file.
    fn Try_on_concerned_file_systems<F, T>(
        &self,
        Path: impl AsRef<Path_type>,
        Closure: F,
    ) -> Result_type<T>
    where
        F: Fn(
            File_system_identifier_type,
            &Internal_file_system_type,
            &Path_type,
        ) -> Result_type<T>,
    {
        if !Path.as_ref().Is_valid() {
            return Err(Error_type::Invalid_path);
        }

        let mut Result_score = 0;
        let mut Result: Option<(File_system_identifier_type, &Internal_file_system_type)> = None;

        let File_systems = self.File_systems.read()?;

        // Try with mounted file systems.
        for (File_system_identifier, File_system) in File_systems.iter() {
            if let Some(Mount_point) = &File_system.Mount_point {
                let Mount_point: &Path_type = Mount_point.as_ref();
                if let Some(Relative_path) = Path.as_ref().Strip_prefix_absolute(Mount_point) {
                    let Score = Relative_path.Get_length();
                    if Score > Result_score {
                        Result_score = Score;
                        Result = Some((*File_system_identifier, File_system));
                    }
                }
            }
        }

        // If a file system is found and the file exists, return the result of the closure.
        if let Some((File_system_identifier, File_system)) = Result {
            match Closure(File_system_identifier, File_system, Path.as_ref()) {
                Ok(Result) => return Ok(Result),
                Err(Error_type::Not_found) => (), // Continue when the file is not found.
                Err(Error) => return Err(Error),
            }
        }

        // Try with special file systems.
        for (File_system_identifier, File_system) in File_systems.iter() {
            if File_system.Mount_point.is_none() {
                match Closure(*File_system_identifier, File_system, Path.as_ref()) {
                    Ok(Result) => return Ok(Result),
                    Err(Error_type::Not_found) => (), // Continue when the file is not found.
                    Err(Error) => return Err(Error),
                }
            }
        }

        Err(Error_type::Not_found)
    }

    fn Get_file_system_from_identifier(
        File_systems: &BTreeMap<File_system_identifier_type, Internal_file_system_type>,
        File_system_identifier: File_system_identifier_type,
    ) -> Result_type<&Internal_file_system_type> {
        File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_identifier)
    }

    /// Mount a file system at a given mount point.
    pub fn Mount(
        &self,
        File_system: Box<dyn File_system_traits>,
        Mount_point: impl AsRef<Path_type>,
    ) -> Result_type<File_system_identifier_type> {
        if !Mount_point.as_ref().Is_valid() {
            return Err(Error_type::Invalid_path);
        }

        let Mount_point = Mount_point.as_ref();

        if !Mount_point.Is_absolute() {
            return Err(Error_type::Invalid_path);
        }

        if self.Exists(Mount_point)? {
            return Err(Error_type::Already_exists);
        }

        let mut File_systems = self.File_systems.write()?;

        let File_system_identifier = Self::Get_new_file_system_identifier(&File_systems)
            .ok_or(Error_type::Too_many_mounted_file_systems)?;

        File_systems.insert(
            File_system_identifier,
            Internal_file_system_type {
                Mount_point: Some(Mount_point.to_owned()),
                Inner: File_system,
            },
        );

        Ok(File_system_identifier)
    }

    /// Unmount a file system and return the file system.
    pub fn Unmount(
        &self,
        File_system_identifier: File_system_identifier_type,
    ) -> Result_type<Box<dyn File_system_traits>> {
        let Internal_file_system = self
            .File_systems
            .write()?
            .remove(&File_system_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(Internal_file_system.Inner)
    }

    fn Get_file_system_from_mount_point<'b>(
        File_systems: &'b BTreeMap<File_system_identifier_type, Internal_file_system_type>,
        Path: &'b dyn AsRef<Path_type>,
    ) -> Result_type<(
        File_system_identifier_type,
        &'b dyn File_system_traits,
        &'b Path_type,
    )> {
        let mut Result_score = 0;
        let mut Result: Option<(
            File_system_identifier_type,
            &'b dyn File_system_traits,
            &'b Path_type,
        )> = None;

        for (File_system_identifier, File_system) in File_systems.iter() {
            if let Some(Mount_point) = &File_system.Mount_point {
                if let Some(Relative_path) = Path.as_ref().Strip_prefix_absolute(Mount_point) {
                    let Score = Relative_path.Get_length();
                    if Score > Result_score {
                        Result_score = Score;
                        Result = Some((
                            *File_system_identifier,
                            File_system.Inner.as_ref(),
                            Relative_path,
                        ));
                    }
                }
            }
        }

        Result.ok_or(Error_type::Invalid_path)
    }

    pub fn Open(
        &self,
        Path: impl AsRef<Path_type>,
        Flags: Flags_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        // - Open file
        self.Try_on_concerned_file_systems(
            Path,
            |File_system_identifier, File_system, Relative_path| {
                File_system
                    .Inner
                    .Open(Task_identifier, &Relative_path, Flags)
                    .map(|File_identifier| {
                        Unique_file_identifier_type::New(File_system_identifier, File_identifier)
                    })
            },
        )
    }

    fn Check_permission(
        &self,
        _: &dyn File_system_traits,
        _: Task_identifier_type,
        _: impl AsRef<Path_type>,
        _: Permission_type,
    ) -> Result_type<()> {
        //let File_permission = self.Get_statistics(File_system, Task_identifier, Relative_path)?;

        //if !File_permission.Include(Permission) {
        //    return Err(Error_type::Permission_denied);
        //}

        Ok(())
    }

    pub fn Close(
        &self,
        File: Unique_file_identifier_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<()> {
        let (File_system_identifier, File_identifier) = File.Split();

        let File_systems = self.File_systems.read()?; // Get the file systems

        Self::Get_file_system_from_identifier(&File_systems, File_system_identifier)?
            .Inner
            .Close(Task_identifier, File_identifier)
    }

    pub fn Read(
        &self,
        File_identifier: Unique_file_identifier_type,
        Buffer: &mut [u8],
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (File_system_identifier, File_identifier) = File_identifier.Split();

        let File_systems = self.File_systems.read()?; // Get the file systems

        Self::Get_file_system_from_identifier(&File_systems, File_system_identifier)?
            .Inner
            .Read(Task_identifier, File_identifier, Buffer)
    }

    pub fn Write(
        &self,
        File: Unique_file_identifier_type,
        Buffer: &[u8],
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (File_system_identifier, File_identifier) = File.Split();

        let File_systems = self.File_systems.read()?; // Get the file systems

        Self::Get_file_system_from_identifier(&File_systems, File_system_identifier)?
            .Inner
            .Write(Task_identifier, File_identifier, Buffer)
    }

    pub fn Set_position(
        &self,
        File_identifier: Unique_file_identifier_type,
        Position: &Position_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (File_system_identifier, File_identifier) = File_identifier.Split();

        let File_systems = self.File_systems.read()?; // Get the file systems

        Self::Get_file_system_from_identifier(&File_systems, File_system_identifier)?
            .Inner
            .Set_position(Task_identifier, File_identifier, Position)
    }

    pub fn Exists(&self, Path: impl AsRef<Path_type>) -> Result_type<bool> {
        match self.Try_on_concerned_file_systems(Path, |_, File_system, Relative_path| {
            File_system.Inner.Exists(&Relative_path)
        }) {
            Ok(Exists) => Ok(Exists),
            Err(Error_type::Not_found) => Ok(false),
            Err(Error) => Err(Error),
        }
    }

    pub fn Set_owner(
        &self,
        Path: impl AsRef<Path_type>,
        User: Option<User_identifier_type>,
        Group: Option<Group_identifier_type>,
    ) -> Result_type<()> {
        self.Try_on_concerned_file_systems(Path, |_, File_system, Relative_path| {
            File_system.Inner.Set_owner(&Relative_path, User, Group)
        })
    }

    pub fn Set_permissions(
        &self,
        Path: impl AsRef<Path_type>,
        Permissions: Permissions_type,
    ) -> Result_type<()> {
        self.Try_on_concerned_file_systems(Path, |_, File_system, Relative_path| {
            File_system
                .Inner
                .Set_permissions(&Relative_path, Permissions)
        })
    }

    pub fn Close_all(&self, Task_identifier: Task_identifier_type) -> Result_type<()> {
        let File_systems = self.File_systems.read()?; // Get the file systems

        for File_system in File_systems.values() {
            File_system.Inner.Close_all(Task_identifier)?;
        }

        Ok(())
    }

    pub fn Add_device(
        &self,
        Path: &'static dyn AsRef<Path_type>,
        Device: Box<dyn Device_trait>,
    ) -> Result_type<()> {
        if !Path.as_ref().Is_valid() {
            return Err(Error_type::Invalid_path);
        }

        let File_systems = self.File_systems.read()?; // Get the file systems

        let File_system = Self::Get_file_system_from_identifier(
            &File_systems,
            Self::Device_file_system_identifier,
        )?;

        File_system.Inner.Add_device(Path, Device)
    }

    pub fn Create_named_pipe(
        &self,
        Path: &impl AsRef<Path_type>,
        Size: Size_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<()> {
        if !Path.as_ref().Is_valid() {
            return Err(Error_type::Invalid_path);
        }

        let mut File_systems = self.File_systems.write()?; // Get the file systems

        let Parent_path = Path.as_ref().Go_parent().ok_or(Error_type::Invalid_path)?;

        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_mount_point(&File_systems, &Parent_path)?; // Get the file system identifier and the relative path

        self.Check_permission(
            File_system,
            Task_identifier,
            Relative_path,
            Permission_type::New_write(),
        )?;

        let User = self.Task_manager.Get_owner(Task_identifier)?;

        let Group = self
            .User_manager
            .Get_user_primary_group(User)
            .map_err(|_| Error_type::Failed_to_get_users_informations)?;

        let Permissions = Permissions_type::New_default(Type_type::Pipe);

        File_systems
            .get_mut(&Self::Pipe_file_system_identifier)
            .ok_or(Error_type::Invalid_path)?
            .Inner
            .Create_named_pipe(Path, Size, User, Group, Permissions)
    }

    pub fn Create_unnamed_pipe(
        &self,
        Size: Size_type,
        Status: Status_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<(Unique_file_identifier_type, Unique_file_identifier_type)> {
        let User = self.Task_manager.Get_owner(Task_identifier)?;

        let Group = self
            .User_manager
            .Get_user_primary_group(User)
            .map_err(|_| Error_type::Failed_to_get_users_informations)?;

        let Permission = Permissions_type::New_default(Type_type::Pipe);

        let (Read, Write) = self
            .File_systems
            .write()?
            .get_mut(&Self::Pipe_file_system_identifier)
            .ok_or(Error_type::Invalid_path)?
            .Inner
            .Create_unnamed_pipe(Task_identifier, Size, Status, User, Group, Permission)?;

        Ok((
            Unique_file_identifier_type::New(Self::Pipe_file_system_identifier, Read),
            Unique_file_identifier_type::New(Self::Pipe_file_system_identifier, Write),
        ))
    }

    pub fn Create_file(
        &self,
        Path: impl AsRef<Path_type>,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<()> {
        let File_systems = self.File_systems.read()?; // Get the file systems

        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_mount_point(&File_systems, &Path)?; // Get the file system identifier and the relative path

        // Check if the user has the right to create the file (write permission on the parent directory)
        self.Check_permission(
            File_system,
            Task_identifier,
            Relative_path.Go_parent().unwrap_or(Path_type::Get_root()),
            Permission_type::New_write(),
        )?;

        File_system.Create_file(&Relative_path)
    }

    pub fn Create_directory(
        &self,
        Path: impl AsRef<Path_type>,
        Recursive: bool,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<()> {
        if Recursive {
            // If the directory already exists, return Ok(()) (only if recursive is true).
            if self.Exists(Path.as_ref())? {
                return Ok(());
            }

            // Create the parent directory recursively.
            self.Create_directory(
                Path.as_ref().Go_parent().ok_or(Error_type::Invalid_path)?,
                true,
                Task_identifier,
            )?
        }

        // Create current directory.
        let File_systems = self.File_systems.read()?; // Get the file systems

        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_mount_point(&File_systems, &Path)?;

        // Check if the user has the right to create the directory (write permission and execute permission on the parent directory)
        self.Check_permission(
            File_system,
            Task_identifier,
            Relative_path,
            Permission_type::New_write_execute(),
        )?;

        File_system.Create_directory(&Relative_path)
    }

    pub fn Delete(
        &self,
        Path: impl AsRef<Path_type>,
        Recursive: bool,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<()> {
        if Recursive {
            todo!()
        }

        // Delete current directory / file.
        self.Try_on_concerned_file_systems(Path.as_ref(), |_, File_system, Relative_path| {
            // Check if the user has the right to delete the file (write permission on the parent directory)
            self.Check_permission(
                &*File_system.Inner,
                Task_identifier,
                Relative_path.Go_parent().ok_or(Error_type::Invalid_path)?,
                Permission_type::New_write(),
            )
        })?;

        self.Try_on_concerned_file_systems(Path, |_, File_system, Relative_path| {
            // Delete the file
            File_system.Inner.Delete(&Relative_path)
        })
    }

    pub fn Transfert_file(
        &self,
        File: Unique_file_identifier_type,
        Current_task: Task_identifier_type,
        New_task: Task_identifier_type,
        New_file_identifier: Option<File_identifier_type>,
    ) -> Result_type<Unique_file_identifier_type> {
        let (File_system_identifier, File_identifier) = File.Split();

        let File_systems = self.File_systems.read()?; // Get the file systems

        let New_file_identifier =
            Self::Get_file_system_from_identifier(&File_systems, File_system_identifier)?
                .Inner
                .Transfert_file_identifier(
                    Current_task,
                    New_task,
                    File_identifier,
                    New_file_identifier,
                )?;

        Ok(Unique_file_identifier_type::New(
            File_system_identifier,
            New_file_identifier,
        ))
    }

    pub fn Flush(
        &self,
        File: Unique_file_identifier_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<()> {
        let (File_system_identifier, File_identifier) = File.Split();

        let File_systems = self.File_systems.read()?; // Get the file systems

        Self::Get_file_system_from_identifier(&File_systems, File_system_identifier)?
            .Inner
            .Flush(Task_identifier, File_identifier)
    }

    pub fn Get_statistics(
        &self,
        File: Unique_file_identifier_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Statistics_type> {
        let (File_system_identifier, File_identifier) = File.Split();

        let File_systems = self.File_systems.read()?; // Get the file systems

        let File_system =
            Self::Get_file_system_from_identifier(&File_systems, File_system_identifier)?;

        File_system
            .Inner
            .Get_statistics(Task_identifier, File_identifier, File_system_identifier)
    }

    pub fn Get_mode(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
    ) -> Result_type<Mode_type> {
        let (File_system_identifier, File_identifier) = File.Split();

        let File_systems = self.File_systems.read()?; // Get the file systems

        let File_system =
            Self::Get_file_system_from_identifier(&File_systems, File_system_identifier)?;

        File_system.Inner.Get_mode(Task, File_identifier)
    }

    pub fn Duplicate_file_identifier(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let (File_system_identifier, File_identifier) = File.Split();

        let File_systems = self.File_systems.read()?;

        let File_system =
            Self::Get_file_system_from_identifier(&File_systems, File_system_identifier)?;

        let File_identifier = File_system
            .Inner
            .Duplicate_file_identifier(Task, File_identifier)?;

        Ok(Unique_file_identifier_type::New(
            File_system_identifier,
            File_identifier,
        ))
    }

    pub fn Create_new_task_standard_io(
        self,
        Standard_in: Unique_file_identifier_type,
        Standard_error: Unique_file_identifier_type,
        Standard_out: Unique_file_identifier_type,
        Current_task: Task_identifier_type,
        New_task: Task_identifier_type,
        Duplicate: bool,
    ) -> Result_type<(
        Unique_file_identifier_type,
        Unique_file_identifier_type,
        Unique_file_identifier_type,
    )> {
        let (Standard_in, Standard_error, Standard_out) = if Duplicate {
            let Standard_in = self.Duplicate_file_identifier(Standard_in, Current_task)?;
            let Standard_error = self.Duplicate_file_identifier(Standard_error, Current_task)?;
            let Standard_out = self.Duplicate_file_identifier(Standard_out, Current_task)?;

            (Standard_in, Standard_error, Standard_out)
        } else {
            (Standard_in, Standard_error, Standard_out)
        };

        let Standard_in = self.Transfert_file(Standard_in, Current_task, New_task, None)?;
        let Standard_error = self.Transfert_file(Standard_error, Current_task, New_task, None)?;
        let Standard_out = self.Transfert_file(Standard_out, Current_task, New_task, None)?;

        Ok((Standard_in, Standard_error, Standard_out))
    }
}
