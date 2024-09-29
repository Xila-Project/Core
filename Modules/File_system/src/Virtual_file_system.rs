use std::{collections::BTreeMap, fmt::Error, path::Path, sync::RwLock};

use Task::Task_identifier_type;
use Users::{Group_identifier_type, User_identifier_type};

use crate::{
    Device::{self, Device_type}, File_identifier_type, File_system, Local_file_identifier_type, Mode_type, Open_type,
    Pipe, Statistics_type,
};

use super::{
    Error_type, File_system_identifier_type, File_system_traits, Flags_type, Path_owned_type,
    Path_type, Permission_type, Permissions_type, Position_type, Result_type, Size_type,
    Status_type, Type_type, Unique_file_identifier_type,
};

struct Internal_file_system_type {
    pub Mount_point: Path_owned_type,
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

        Virtual_file_system_instance.replace(Virtual_file_system_type::New(
            Task::Get_instance(),
            Users::Get_instance(),
            Time::Get_instance(),
        )?);

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
    /// Devices.
    Device_file_system: Device::File_system_type,
    /// Pipes.
    Pipe_file_system: Pipe::File_system_type,
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
        _: &'static Time::Manager_type,
    ) -> Result_type<Self> {
        let File_systems = BTreeMap::new();

        //        let Pipe_file_system = Pipe::File_system_type::New();
        //
        //        File_systems.insert(
        //            Self::Pipe_file_system_identifier,
        //            Internal_file_system_type {
        //                Mount_point: None,
        //                Inner: Box::new(Pipe_file_system),
        //            },
        //        );
        //
        //        let Device_file_system = Device::File_system_type::New();
        //
        //        File_systems.insert(
        //            Self::Device_file_system_identifier,
        //            Internal_file_system_type {
        //                Mount_point: None,
        //                Inner: Box::new(Device_file_system),
        //            },
        //        );

        Ok(Self {
            Task_manager,
            User_manager,
            File_systems: RwLock::new(File_systems),
            Device_file_system: Device::File_system_type::New(),
            Pipe_file_system: Pipe::File_system_type::New(),
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
            let Mount_point: &Path_type = File_system.Mount_point.as_ref();
            if let Some(Relative_path) = Path.as_ref().Strip_prefix_absolute(Mount_point) {
                let Score = Relative_path.Get_length();
                if Score > Result_score {
                    Result_score = Score;
                    Result = Some((*File_system_identifier, File_system));
                }
            }
        }

        // If a file system is found and the file exists, return the result of the closure.
        if let Some((File_system_identifier, File_system)) = Result {
            Closure(File_system_identifier, File_system, Path.as_ref())?;
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
        Task: Task_identifier_type,
    ) -> Result_type<File_system_identifier_type> {
        if !Mount_point.as_ref().Is_valid() {
            return Err(Error_type::Invalid_path);
        }

        let Mount_point = Mount_point.as_ref();

        if !Mount_point.Is_absolute() {
            return Err(Error_type::Invalid_path);
        }

        let mut File_systems = self.File_systems.write()?; // Get the file systems

        // Create the special file for mounting the file system.
        let Flags = Flags_type::New(
            Mode_type::Read_write,
            Some(Open_type::Create_only.Set_directory(true)),
            None,
        );
        let (_, Parent_file_system, _) =
            Self::Get_file_system_from_path(&File_systems, &Mount_point)?;

        let Directory = Parent_file_system.Open(Task, &Mount_point, Flags)?;

        let File_system_identifier = Self::Get_new_file_system_identifier(&File_systems)
            .ok_or(Error_type::Too_many_mounted_file_systems)?;

        File_systems.insert(
            File_system_identifier,
            Internal_file_system_type {
                Mount_point: Mount_point.to_owned(),
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

    fn Get_file_system_from_path<'b>(
        File_systems: &'b BTreeMap<File_system_identifier_type, Internal_file_system_type>,
        Path: &'b impl AsRef<Path_type>,
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

        let Path = Path.as_ref();

        for (File_system_identifier, File_system) in File_systems.iter() {
            let Mount_point = File_system.Mount_point.as_ref();
            if let Some(Relative_path) = Path.Strip_prefix_absolute(Mount_point) {
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

        Result.ok_or(Error_type::Invalid_path)
    }

    pub fn Open(
        &self,
        Path: &impl AsRef<Path_type>,
        Flags: Flags_type,
        Task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let File_systems = self.File_systems.read()?; // Get the file systems

        let (File_system_identifier, File_system, Relative_path) =
            Self::Get_file_system_from_path(&File_systems, Path)?; // Get the file system identifier and the relative path

        let Local_file_identifier = File_system.Open(Task, &Relative_path, Flags)?;

        let (_, Unique_file_identifier) =
            Local_file_identifier.Into_unique_file_identifier(File_system_identifier);

        Ok(Unique_file_identifier)
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
        Task: Task_identifier_type,
    ) -> Result_type<()> {
        let (File_system, Local_file_identifier) = File.Into_local_file_identifier(Task);

        self.File_systems
            .read()?
            .get(&File_system)
            .ok_or(Error_type::Invalid_identifier)?
            .Inner
            .Close(Local_file_identifier)
    }

    pub fn Read(
        &self,
        File: Unique_file_identifier_type,
        Buffer: &mut [u8],
        Task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (File_system, Local_file_identifier) = File.Into_local_file_identifier(Task);

        self.File_systems
            .read()?
            .get(&File_system)
            .ok_or(Error_type::Invalid_identifier)?
            .Inner
            .Read(Local_file_identifier, Buffer)
    }

    pub fn Write(
        &self,
        File: Unique_file_identifier_type,
        Buffer: &[u8],
        Task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (File_system, Local_file_identifier) = File.Into_local_file_identifier(Task);

        self.File_systems
            .read()?
            .get(&File_system)
            .ok_or(Error_type::Invalid_identifier)?
            .Inner
            .Write(Local_file_identifier, Buffer)
    }

    pub fn Set_position(
        &self,
        File: Unique_file_identifier_type,
        Position: &Position_type,
        Task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (File_system, Local_file_identifier) = File.Into_local_file_identifier(Task);

        self.File_systems
            .read()?
            .get(&File_system)
            .ok_or(Error_type::Invalid_identifier)?
            .Inner
            .Set_position(Local_file_identifier, Position)
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
        Device: Device_type,
    ) -> Result_type<()> {
        todo!()
    }

    pub fn Create_named_pipe(
        &self,
        Path: &impl AsRef<Path_type>,
        Size: Size_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<()> {
        todo!();
    }

    pub fn Create_unnamed_pipe(
        &self,
        Size: Size_type,
        Status: Status_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<(Unique_file_identifier_type, Unique_file_identifier_type)> {
        todo!()
    }

    pub fn Delete(&self, Path: impl AsRef<Path_type>) -> Result_type<()> {
        let File_systems = self.File_systems.read()?; // Get the file systems

        Self::Get_file_system_from_path(&File_systems, &Path)?
            .1
            .Delete(&Path.as_ref())
    }

    pub fn Transfert_file_identifier(
        &self,
        File: Unique_file_identifier_type,
        Current_task: Task_identifier_type,
        New_task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let (File_system, File) = File.Into_local_file_identifier(Current_task);

        let File_systems = self.File_systems.read()?;

        let New_file = Self::Get_file_system_from_identifier(&File_systems, File_system)?
            .Inner
            .Transfert_file_identifier(New_task, File)?;

        let (_, New_file) = New_file.Into_unique_file_identifier(File_system);

        Ok(New_file)
    }

    pub fn Flush(
        &self,
        File: Unique_file_identifier_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<()> {
        let (File_system, File_identifier) = File.Into_local_file_identifier(Task_identifier);

        self.File_systems
            .read()?
            .get(&File_system)
            .ok_or(Error_type::Invalid_identifier)?
            .Inner
            .Flush(File_identifier)
    }

    pub fn Get_statistics(
        &self,
        File: Unique_file_identifier_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Statistics_type> {
        let (File_system, File) = File.Into_local_file_identifier(Task_identifier);

        let File_systems = self.File_systems.read()?;

        Self::Get_file_system_from_identifier(&File_systems, File_system)?
            .Inner
            .Get_statistics(File)
    }

    pub fn Get_mode(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
    ) -> Result_type<Mode_type> {
        let (File_system, File) = File.Into_local_file_identifier(Task);

        let File_systems = self.File_systems.read()?;

        Self::Get_file_system_from_identifier(&File_systems, File_system)?
            .Inner
            .Get_mode(File)
    }

    pub fn Duplicate_file_identifier(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let (File_system, File) = File.Into_local_file_identifier(Task);

        let File_systems = self.File_systems.read()?;

        let New_file = Self::Get_file_system_from_identifier(&File_systems, File_system)?
            .Inner
            .Duplicate_file_identifier(File)?;

        let (_, New_file) = New_file.Into_unique_file_identifier(File_system);

        Ok(New_file)
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

        let Standard_in = self.Transfert_file_identifier(Standard_in, Current_task, New_task)?;
        let Standard_error =
            self.Transfert_file_identifier(Standard_error, Current_task, New_task)?;
        let Standard_out = self.Transfert_file_identifier(Standard_out, Current_task, New_task)?;

        Ok((Standard_in, Standard_error, Standard_out))
    }
}
