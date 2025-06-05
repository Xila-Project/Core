use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use Synchronization::{
    blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, rwlock::RwLock,
};

use alloc::{boxed::Box, collections::BTreeMap};

use Network::{IP_type, Network_socket_driver_trait, Port_type, Protocol_type};
use Task::Task_identifier_type;
use Time::Duration_type;
use Users::{Group_identifier_type, User_identifier_type};

use File_system::{
    Device_type, Entry_type, File_identifier_type, Local_file_identifier_type, Metadata_type,
    Mode_type, Open_type, Statistics_type, Time_type, Type_type,
};

use File_system::{
    Error_type, File_system_identifier_type, File_system_traits, Flags_type, Path_owned_type,
    Path_type, Permissions_type, Position_type, Result_type, Size_type, Status_type,
    Unique_file_identifier_type,
};

use crate::Device::Internal_path_type;
use crate::{Device, Pipe, Socket_address_type};

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
static Virtual_file_system_instance: OnceLock<Virtual_file_system_type> = OnceLock::new();

pub fn Initialize(
    Root_file_system: Box<dyn File_system_traits>,
    Network_socket_driver: Option<&'static dyn Network_socket_driver_trait>,
) -> Result<&'static Virtual_file_system_type<'static>, crate::Error_type> {
    let Virtual_file_system = Virtual_file_system_type::New(
        Task::Get_instance(),
        Users::Get_instance(),
        Time::Get_instance(),
        Root_file_system,
        Network_socket_driver,
    )?;

    Ok(Virtual_file_system_instance.get_or_init(|| Virtual_file_system))
}

pub fn Get_instance() -> &'static Virtual_file_system_type<'static> {
    Virtual_file_system_instance
        .try_get()
        .expect("Virtual file system not initialized")
}

/// The virtual file system.
///
/// It is a singleton.
pub struct Virtual_file_system_type<'a> {
    /// Mounted file systems.
    File_systems: RwLock<
        CriticalSectionRawMutex,
        BTreeMap<File_system_identifier_type, Internal_file_system_type>,
    >,
    /// Devices.
    Device_file_system: Device::File_system_type<'a>,
    /// Pipes.
    Pipe_file_system: Pipe::File_system_type,
    /// Network sockets.
    Network_socket_driver: Option<&'a dyn Network_socket_driver_trait>,
}

impl<'a> Virtual_file_system_type<'a> {
    pub const Standard_input_file_identifier: File_identifier_type = File_identifier_type::New(0);
    pub const Standard_output_file_identifier: File_identifier_type = File_identifier_type::New(1);
    pub const Standard_error_file_identifier: File_identifier_type = File_identifier_type::New(2);

    pub fn New(
        _: &'static Task::Manager_type,
        _: &'static Users::Manager_type,
        _: &'static Time::Manager_type,
        Root_file_system: Box<dyn File_system_traits>,
        Network_socket_driver: Option<&'a dyn Network_socket_driver_trait>,
    ) -> Result_type<Self> {
        let mut File_systems = BTreeMap::new();

        let Identifier = Self::Get_new_file_system_identifier(&File_systems)
            .ok_or(Error_type::Too_many_mounted_file_systems)?;

        File_systems.insert(
            Identifier,
            Internal_file_system_type {
                Mount_point: Path_owned_type::New("/".to_string()).unwrap(),
                Inner: Root_file_system,
            },
        );

        Ok(Self {
            File_systems: RwLock::new(File_systems),
            Device_file_system: Device::File_system_type::New(),
            Pipe_file_system: Pipe::File_system_type::New(),
            Network_socket_driver,
        })
    }

    pub async fn Uninitialize(&self) {
        if let Ok(Inodes) = self
            .Device_file_system
            .Get_devices_from_path(Path_type::Root)
            .await
        {
            for Inode in Inodes {
                if let Ok(Path) = self.Device_file_system.Get_path_from_inode(Inode).await {
                    match Path {
                        Internal_path_type::Owned(Path) => {
                            let _ = self.Remove(Path).await;
                        }
                        Internal_path_type::Borrowed(Path) => {
                            let _ = self.Remove(Path).await;
                        }
                    }
                }
            }
        }
    }

    fn Get_new_file_system_identifier(
        File_systems: &BTreeMap<File_system_identifier_type, Internal_file_system_type>,
    ) -> Option<File_system_identifier_type> {
        let mut File_system_identifier = File_system_identifier_type::Minimum;

        while File_systems.contains_key(&File_system_identifier) {
            File_system_identifier += 1;
        }

        Some(File_system_identifier)
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
    pub async fn Mount_file_system(
        &self,
        File_system: Box<dyn File_system_traits>,
        Path: impl AsRef<Path_type>,
        Task: Task_identifier_type,
    ) -> Result_type<File_system_identifier_type> {
        if !Path.as_ref().Is_valid() {
            return Err(Error_type::Invalid_path);
        }

        let Path = Path.as_ref();

        if !Path.Is_absolute() {
            return Err(Error_type::Invalid_path);
        }

        let mut File_systems = self.File_systems.write().await; // Get the file systems

        // Create a directory in the underlying file system
        let (_, Parent_file_system, Relative_path) =
            Self::Get_file_system_from_path(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(Task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        Parent_file_system.Create_directory(Relative_path, Time, User, Group)?;

        // Create a directory at the mount point
        let File_system_identifier = Self::Get_new_file_system_identifier(&File_systems)
            .ok_or(Error_type::Too_many_mounted_file_systems)?;

        File_systems.insert(
            File_system_identifier,
            Internal_file_system_type {
                Mount_point: Path.to_owned(),
                Inner: File_system,
            },
        );

        Ok(File_system_identifier)
    }

    pub async fn Unmount_file_system(
        &self,
        Path: impl AsRef<Path_type>,
        Task: Task_identifier_type,
    ) -> Result_type<()> {
        let Path = Path.as_ref();

        if !Path.Is_valid() || !Path.Is_absolute() {
            return Err(Error_type::Invalid_path);
        }

        let mut File_systems = self.File_systems.write().await; // Get the file systems

        let File_system_identifier = {
            let (File_system_identifier, _, Relative_path) =
                Self::Get_file_system_from_path(&File_systems, &Path)?; // Get the file system identifier and the relative path

            if !Relative_path.Is_root() {
                return Err(Error_type::Invalid_path);
            }

            File_system_identifier
        };

        let File_system = File_systems
            .remove(&File_system_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        File_system.Inner.Close_all(Task)?;

        let (_, Parent_file_system, Relative_path) =
            Self::Get_file_system_from_path(&File_systems, &File_system.Mount_point)?;

        Parent_file_system.Remove(Relative_path)?;

        Ok(())
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
        let mut Result: Option<File_system_identifier_type> = None;

        let Path = Path.as_ref();
        let Path_components = Path.Get_components();

        for (File_system_identifier, File_system) in File_systems.iter() {
            let Mount_point: &Path_type = File_system.Mount_point.as_ref();
            let Mount_point_components = Mount_point.Get_components();

            let Score = Path_components
                .clone()
                .Get_common_components(Mount_point_components);

            if Result_score < Score {
                Result_score = Score;
                Result = Some(*File_system_identifier);
            }
        }

        let File_system_identifier = Result.ok_or(Error_type::Invalid_path)?;

        let File_system = File_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?;

        let Relative_path = Path
            .Strip_prefix_absolute(File_system.Mount_point.as_ref())
            .ok_or(Error_type::Invalid_path)?;

        Ok((
            File_system_identifier,
            File_system.Inner.as_ref(),
            Relative_path,
        ))
    }

    pub async fn Open(
        &self,
        Path: &impl AsRef<Path_type>,
        Flags: Flags_type,
        Task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let File_systems = self.File_systems.read().await; // Get the file systems

        let (File_system_identifier, File_system, Relative_path) =
            Self::Get_file_system_from_path(&File_systems, Path)?; // Get the file system identifier and the relative path

        let Time: Time_type = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(Task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        let Local_file = File_system.Open(Task, Relative_path, Flags, Time, User, Group)?;

        let Metadata = File_system.Get_metadata(Local_file)?;

        let (_, Unique_file) = Local_file.Into_unique_file_identifier(File_system_identifier);

        let Unique_file = match Metadata.Get_type() {
            Type_type::Character_device | Type_type::Block_device => {
                if let Some(Inode) = Metadata.Get_inode() {
                    let Local_file = self
                        .Device_file_system
                        .Open(Inode, Task, Flags, Unique_file)
                        .await?;

                    Local_file
                        .Into_unique_file_identifier(
                            File_system_identifier_type::Device_file_system,
                        )
                        .1
                } else {
                    return Err(Error_type::Corrupted)?;
                }
            }
            Type_type::Pipe => {
                if let Some(Inode) = Metadata.Get_inode() {
                    let Local_file = self
                        .Pipe_file_system
                        .Open(Inode, Task, Flags, Unique_file)
                        .await?;

                    Local_file
                        .Into_unique_file_identifier(File_system_identifier_type::Pipe_file_system)
                        .1
                } else {
                    return Err(Error_type::Corrupted)?;
                }
            }
            _ => Unique_file,
        };

        Ok(Unique_file)
    }

    pub async fn Close(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
    ) -> crate::Result_type<()> {
        let (File_system, Local_file) = File.Into_local_file_identifier(Task);

        let Underlying_file = match File_system {
            File_system_identifier_type::Pipe_file_system => {
                match self.Pipe_file_system.Close(Local_file).await? {
                    Some(Underlying_file) => Underlying_file,
                    None => return Ok(()),
                }
            }
            File_system_identifier_type::Device_file_system => {
                self.Device_file_system.Close(Local_file).await?
            }
            File_system_identifier_type::Network_socket_file_system => {
                self.Network_socket_driver
                    .ok_or(Error_type::Unsupported_operation)?
                    .Close(Local_file)?;

                return Ok(());
            }
            _ => {
                self.File_systems
                    .read()
                    .await
                    .get(&File_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .Inner
                    .Close(Local_file)?;

                return Ok(());
            }
        };

        // - If there is an underlying file (some pipe and devices), close it too.
        let (File_system, Local_file) = Underlying_file.Into_local_file_identifier(Task);

        self.File_systems
            .read()
            .await
            .get(&File_system)
            .ok_or(Error_type::Invalid_identifier)?
            .Inner
            .Close(Local_file)?;

        Ok(())
    }

    pub async fn Read(
        &self,
        File: Unique_file_identifier_type,
        Buffer: &mut [u8],
        Task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (File_system, Local_file_identifier) = File.Into_local_file_identifier(Task);

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let (Size, Underlying_file) = match File_system {
            File_system_identifier_type::Pipe_file_system => {
                self.Pipe_file_system
                    .Read(Local_file_identifier, Buffer)
                    .await?
            }
            File_system_identifier_type::Device_file_system => {
                let Result = self
                    .Device_file_system
                    .Read(Local_file_identifier, Buffer)
                    .await?;
                (Result.0, Some(Result.1))
            }
            _ => {
                return self
                    .File_systems
                    .read()
                    .await
                    .get(&File_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .Inner
                    .Read(Local_file_identifier, Buffer, Time)
            }
        };

        if let Some(Underlying_file) = Underlying_file {
            let (File_system, Local_file_identifier) =
                Underlying_file.Into_local_file_identifier(Task);

            self.File_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Read(Local_file_identifier, &mut [0; 0], Time)?;
        }
        Ok(Size)
    }

    pub async fn Read_line(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
        Buffer: &mut String,
    ) -> Result_type<Size_type> {
        let (File_system, Local_file_identifier) = File.Into_local_file_identifier(Task);

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let (Size, Underlying_file) = match File_system {
            File_system_identifier_type::Pipe_file_system => {
                self.Pipe_file_system
                    .Read_line(Local_file_identifier, Buffer)
                    .await?
            }
            File_system_identifier_type::Device_file_system => {
                let Result = self
                    .Device_file_system
                    .Read_line(Local_file_identifier, Buffer)
                    .await?;
                (Result.0, Some(Result.1))
            }
            _ => {
                return self
                    .File_systems
                    .read()
                    .await
                    .get(&File_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .Inner
                    .Read_line(Local_file_identifier, Buffer, Time)
            }
        };

        if let Some(Underlying_file) = Underlying_file {
            let (File_system, Local_file_identifier) =
                Underlying_file.Into_local_file_identifier(Task);

            self.File_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Read_line(Local_file_identifier, Buffer, Time)?;
        }

        Ok(Size)
    }

    pub async fn Read_to_end(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
        Buffer: &mut Vec<u8>,
    ) -> Result_type<Size_type> {
        const Chunk_size: usize = 512;

        let mut Read_size = 0;

        loop {
            let mut Chunk = vec![0; Chunk_size];

            let Size: usize = self.Read(File, &mut Chunk, Task).await?.into();

            if Size == 0 {
                break;
            }

            Buffer.extend_from_slice(&Chunk[..Size]);

            Read_size += Size;
        }

        Ok(Read_size.into())
    }

    pub async fn Write(
        &self,
        File: Unique_file_identifier_type,
        Buffer: &[u8],
        Task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (File_system, Local_file_identifier) = File.Into_local_file_identifier(Task);

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let (Size, Underlying_file) = match File_system {
            File_system_identifier_type::Pipe_file_system => {
                self.Pipe_file_system
                    .Write(Local_file_identifier, Buffer)
                    .await?
            }
            File_system_identifier_type::Device_file_system => {
                let Result = self
                    .Device_file_system
                    .Write(Local_file_identifier, Buffer)
                    .await?;
                (Result.0, Some(Result.1))
            }
            _ => {
                return self
                    .File_systems
                    .read()
                    .await
                    .get(&File_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .Inner
                    .Write(Local_file_identifier, Buffer, Time)
            }
        };

        if let Some(Underlying_file) = Underlying_file {
            let (File_system, Local_file_identifier) =
                Underlying_file.Into_local_file_identifier(Task);

            self.File_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Write(Local_file_identifier, &[0; 0], Time)?;
        }

        Ok(Size)
    }

    pub async fn Set_position(
        &self,
        File: Unique_file_identifier_type,
        Position: &Position_type,
        Task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (File_system, Local_file) = File.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Pipe_file_system => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::Device_file_system => {
                let (Size, Underlying_file) = self
                    .Device_file_system
                    .Set_position(Local_file, Position)
                    .await?;

                let (File_system, Local_file) = Underlying_file.Into_local_file_identifier(Task);

                self.File_systems
                    .read()
                    .await
                    .get(&File_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .Inner
                    .Set_position(Local_file, Position)?;

                Ok(Size)
            }
            _ => self
                .File_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Set_position(Local_file, Position),
        }
    }

    pub async fn Set_owner(
        &self,
        Path: impl AsRef<Path_type>,
        User: Option<User_identifier_type>,
        Group: Option<Group_identifier_type>,
    ) -> Result_type<()> {
        let File_systems = self.File_systems.read().await; // Get the file systems

        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_path(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let mut Metadata = File_system.Get_metadata_from_path(Relative_path)?;

        if let Some(User) = User {
            Metadata.Set_owner(User);
        }

        if let Some(Group) = Group {
            Metadata.Set_group(Group);
        }

        File_system.Set_metadata_from_path(Relative_path, &Metadata)
    }

    pub async fn Set_permissions(
        &self,
        Path: impl AsRef<Path_type>,
        Permissions: Permissions_type,
    ) -> Result_type<()> {
        let File_systems = self.File_systems.read().await; // Get the file systems

        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_path(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let mut Metadata = File_system.Get_metadata_from_path(Relative_path)?;

        Metadata.Set_permissions(Permissions);

        File_system.Set_metadata_from_path(Relative_path, &Metadata)
    }

    pub async fn Close_all(&self, Task_identifier: Task_identifier_type) -> Result_type<()> {
        let File_systems = self.File_systems.read().await; // Get the file systems

        for File_system in File_systems.values() {
            File_system.Inner.Close_all(Task_identifier)?;
        }

        self.Device_file_system.Close_all(Task_identifier).await?;

        self.Pipe_file_system.Close_all(Task_identifier).await?;

        Ok(())
    }

    pub async fn Mount_device(
        &self,
        Task: Task_identifier_type,
        Path: &impl AsRef<Path_type>,
        Device: Device_type,
    ) -> Result_type<()> {
        let File_systems = self.File_systems.read().await; // Get the file systems

        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_path(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(Task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        let File = File_system.Open(
            Task,
            Relative_path,
            Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None),
            Time,
            User,
            Group,
        )?;

        File_system.Close(File)?;

        let Inode = self
            .Device_file_system
            .Mount_device(Relative_path.to_owned(), Device)
            .await?;

        let Time: Time_type = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(Task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        let mut Metadata = Metadata_type::Get_default(Type_type::Block_device, Time, User, Group)
            .ok_or(Error_type::Invalid_parameter)?;
        Metadata.Set_inode(Inode);

        File_system.Set_metadata_from_path(Relative_path, &Metadata)?;

        Ok(())
    }

    pub async fn Mount_static_device(
        &self,
        Task: Task_identifier_type,
        Path: &'a impl AsRef<Path_type>,
        Device: Device_type,
    ) -> Result_type<()> {
        let File_systems = self.File_systems.read().await; // Get the file systems

        // Create a special file in the underlying file system.
        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_path(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(Task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        let File = File_system.Open(
            Task,
            Relative_path,
            Flags_type::New(Mode_type::Write_only, Some(Open_type::Create_only), None),
            Time,
            User,
            Group,
        )?;

        File_system.Close(File)?;

        let Type = if Device.Is_a_block_device() {
            Type_type::Block_device
        } else {
            Type_type::Character_device
        };

        // Create the actual device.
        let Inode = self
            .Device_file_system
            .Mount_static_device(Path, Device)
            .await?;

        let Time: Time_type = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(Task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        // Set the metadata of the special file.
        let mut Metadata = Metadata_type::Get_default(Type, Time, User, Group)
            .ok_or(Error_type::Invalid_parameter)?;
        Metadata.Set_inode(Inode);

        File_system.Set_metadata_from_path(Relative_path, &Metadata)?;

        Ok(())
    }

    pub async fn Create_named_pipe(
        &self,
        Path: &impl AsRef<Path_type>,
        Size: usize,
        Task: Task_identifier_type,
    ) -> Result_type<()> {
        let File_systems = self.File_systems.read().await; // Get the file systems

        let (_, File_system, Relative_path) = Self::Get_file_system_from_path(&File_systems, Path)?; // Get the file system identifier and the relative path

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(Task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        let File = File_system.Open(
            Task,
            Relative_path,
            Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None),
            Time,
            User,
            Group,
        )?;

        File_system.Close(File)?;

        let Inode = self.Pipe_file_system.Create_named_pipe(Size).await?;

        let Time: Time_type = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(Task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        let mut Metadata = Metadata_type::Get_default(Type_type::Pipe, Time, User, Group)
            .ok_or(Error_type::Invalid_parameter)?;
        Metadata.Set_inode(Inode);

        File_system.Set_metadata_from_path(Relative_path, &Metadata)?;

        Ok(())
    }

    pub async fn Create_unnamed_pipe(
        &self,
        Task: Task_identifier_type,
        Status: Status_type,
        Size: usize,
    ) -> Result_type<(Unique_file_identifier_type, Unique_file_identifier_type)> {
        let (Read, Write) = self
            .Pipe_file_system
            .Create_unnamed_pipe(Task, Status, Size)
            .await?;

        let (_, Read) =
            Read.Into_unique_file_identifier(File_system_identifier_type::Pipe_file_system);
        let (_, Write) =
            Write.Into_unique_file_identifier(File_system_identifier_type::Pipe_file_system);

        Ok((Read, Write))
    }

    pub async fn Remove(&self, Path: impl AsRef<Path_type>) -> Result_type<()> {
        let File_systems = self.File_systems.read().await; // Get the file systems

        // - Check metadata on the underlying file system
        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_path(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let Metadata = File_system.Get_metadata_from_path(Relative_path)?;

        File_system.Remove(Relative_path)?;

        match Metadata.Get_type() {
            Type_type::Pipe => {
                if let Some(Inode) = Metadata.Get_inode() {
                    match self.Pipe_file_system.Remove(Inode).await {
                        Ok(_) | Err(Error_type::Invalid_inode) => (),
                        Err(Error) => {
                            return Err(Error);
                        }
                    }
                }
            }
            Type_type::Block_device => {
                if let Some(Inode) = Metadata.Get_inode() {
                    match self.Device_file_system.Remove(Inode).await {
                        Ok(_) | Err(Error_type::Invalid_inode) => (),
                        Err(Error) => {
                            return Err(Error);
                        }
                    }
                }
            }
            Type_type::Character_device => {
                if let Some(Inode) = Metadata.Get_inode() {
                    match self.Device_file_system.Remove(Inode).await {
                        Ok(_) | Err(Error_type::Invalid_inode) => (),
                        Err(Error) => {
                            return Err(Error);
                        }
                    }
                }
            }

            _ => (),
        };

        Ok(())
    }

    pub async fn Transfert(
        &self,
        File: Unique_file_identifier_type,
        Current_task: Task_identifier_type,
        New_task: Task_identifier_type,
        New_file: Option<File_identifier_type>,
    ) -> Result_type<Unique_file_identifier_type> {
        let (File_system, File) = File.Into_local_file_identifier(Current_task);

        let Underlying_file = match File_system {
            File_system_identifier_type::Pipe_file_system => {
                self.Pipe_file_system.Get_underlying_file(File).await?
            }
            File_system_identifier_type::Device_file_system => {
                Some(self.Device_file_system.Get_underlying_file(File).await?)
            }
            _ => None,
        };

        let File_systems = self.File_systems.read().await;

        let Underlying_file = if let Some(Underlying_file) = Underlying_file {
            let (File_system, Local_file) =
                Underlying_file.Into_local_file_identifier(Current_task);

            Some(
                File_systems
                    .get(&File_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .Inner
                    .Transfert(New_task, Local_file, New_file)?
                    .Into_unique_file_identifier(File_system)
                    .1,
            )
        } else {
            None
        };

        let New_file = match File_system {
            File_system_identifier_type::Pipe_file_system => {
                self.Pipe_file_system
                    .Transfert(New_task, File, New_file)
                    .await?
            }
            File_system_identifier_type::Device_file_system => {
                let Underlying_file = Underlying_file.ok_or(Error_type::Internal_error)?;

                self.Device_file_system
                    .Transfert(New_task, File, Underlying_file, New_file)
                    .await?
            }
            _ => Self::Get_file_system_from_identifier(&File_systems, File_system)?
                .Inner
                .Transfert(New_task, File, New_file)?,
        };

        let (_, New_file) = New_file.Into_unique_file_identifier(File_system);

        Ok(New_file)
    }

    pub async fn Flush(
        &self,
        File: Unique_file_identifier_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<()> {
        let (File_system, File_identifier) = File.Into_local_file_identifier(Task_identifier);

        if File_system == File_system_identifier_type::Pipe_file_system {
            Ok(())
        } else if File_system == File_system_identifier_type::Device_file_system {
            let Underlying_file = self.Device_file_system.Flush(File_identifier).await?;

            let (File_system, Local_file) =
                Underlying_file.Into_local_file_identifier(Task_identifier);

            self.File_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Flush(Local_file)?;

            Ok(())
        } else {
            self.File_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Flush(File_identifier)
        }
    }

    pub async fn Get_statistics(
        &self,
        File: Unique_file_identifier_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Statistics_type> {
        let (File_system, Local_file) = File.Into_local_file_identifier(Task_identifier);

        let File_systems = self.File_systems.read().await;

        let File = match File_system {
            File_system_identifier_type::Device_file_system => {
                self.Device_file_system
                    .Get_underlying_file(Local_file)
                    .await?
            }
            File_system_identifier_type::Pipe_file_system => self
                .Pipe_file_system
                .Get_underlying_file(Local_file)
                .await?
                .ok_or(Error_type::Unsupported_operation)?,
            _ => File,
        };

        let (File_system, Local_file) = File.Into_local_file_identifier(Task_identifier);

        Self::Get_file_system_from_identifier(&File_systems, File_system)?
            .Inner
            .Get_statistics(Local_file)
    }

    pub async fn Open_directory(
        &self,
        Path: &impl AsRef<Path_type>,
        Task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let File_systems = self.File_systems.read().await; // Get the file systems

        let (File_system_identifier, File_system, Relative_path) =
            Self::Get_file_system_from_path(&File_systems, Path)?; // Get the file system identifier and the relative path

        let (_, File) = File_system
            .Open_directory(Relative_path, Task)?
            .Into_unique_file_identifier(File_system_identifier);

        Ok(File)
    }

    pub async fn Read_directory(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
    ) -> Result_type<Option<Entry_type>> {
        let (File_system, File) = File.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Pipe_file_system => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::Device_file_system => {
                Err(Error_type::Unsupported_operation)
            }
            _ => self
                .File_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Read_directory(File),
        }
    }

    pub async fn Set_position_directory(
        &self,
        File: Unique_file_identifier_type,
        Position: Size_type,
        Task: Task_identifier_type,
    ) -> Result_type<()> {
        let (File_system, File) = File.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Pipe_file_system => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::Device_file_system => {
                Err(Error_type::Unsupported_operation)
            }
            _ => self
                .File_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Set_position_directory(File, Position),
        }
    }

    pub async fn Get_position_directory(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (File_system, File) = File.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Pipe_file_system => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::Device_file_system => {
                Err(Error_type::Unsupported_operation)
            }
            _ => self
                .File_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Get_position_directory(File),
        }
    }

    pub async fn Rewind_directory(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
    ) -> Result_type<()> {
        let (File_system, File) = File.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Pipe_file_system => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::Device_file_system => {
                Err(Error_type::Unsupported_operation)
            }
            _ => self
                .File_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Rewind_directory(File),
        }
    }

    pub async fn Close_directory(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
    ) -> Result_type<()> {
        let (File_system, File) = File.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Pipe_file_system => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::Device_file_system => {
                Err(Error_type::Unsupported_operation)
            }
            _ => self
                .File_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Close_directory(File),
        }
    }

    pub async fn Create_directory(
        &self,
        Path: &impl AsRef<Path_type>,
        Task: Task_identifier_type,
    ) -> Result_type<()> {
        let File_systems = self.File_systems.read().await; // Get the file systems

        let (_, File_system, Relative_path) = Self::Get_file_system_from_path(&File_systems, Path)?; // Get the file system identifier and the relative path

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(Task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        File_system.Create_directory(Relative_path, Time, User, Group)
    }

    pub async fn Get_mode(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
    ) -> Result_type<Mode_type> {
        let (File_system, File) = File.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Pipe_file_system => {
                self.Pipe_file_system.Get_mode(File).await
            }
            File_system_identifier_type::Device_file_system => {
                self.Device_file_system.Get_mode(File).await
            }
            _ => self
                .File_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .Inner
                .Get_mode(File),
        }
    }

    pub async fn Duplicate_file_identifier(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let (File_system, File) = File.Into_local_file_identifier(Task);

        let Underlying_file = match File_system {
            File_system_identifier_type::Pipe_file_system => {
                self.Pipe_file_system.Get_underlying_file(File).await?
            }
            File_system_identifier_type::Device_file_system => {
                Some(self.Device_file_system.Get_underlying_file(File).await?)
            }
            _ => None,
        };

        let File_systems = self.File_systems.read().await;

        let Underlying_file = if let Some(Underlying_file) = Underlying_file {
            let (File_system, Local_file) = Underlying_file.Into_local_file_identifier(Task);

            Some(
                File_systems
                    .get(&File_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .Inner
                    .Duplicate(Local_file)?
                    .Into_unique_file_identifier(File_system)
                    .1,
            )
        } else {
            None
        };

        let New_file = match File_system {
            File_system_identifier_type::Pipe_file_system => {
                self.Pipe_file_system
                    .Duplicate(File, Underlying_file)
                    .await?
            }
            File_system_identifier_type::Device_file_system => {
                let Underlying_file = Underlying_file.ok_or(Error_type::Internal_error)?;

                self.Device_file_system
                    .Duplicate(File, Underlying_file)
                    .await?
            }
            _ => Self::Get_file_system_from_identifier(&File_systems, File_system)?
                .Inner
                .Duplicate(File)?,
        };

        let (_, New_file) = New_file.Into_unique_file_identifier(File_system);

        Ok(New_file)
    }

    pub async fn Create_new_task_standard_io(
        &self,
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
            let Standard_in = self
                .Duplicate_file_identifier(Standard_in, Current_task)
                .await?;
            let Standard_error = self
                .Duplicate_file_identifier(Standard_error, Current_task)
                .await?;
            let Standard_out = self
                .Duplicate_file_identifier(Standard_out, Current_task)
                .await?;

            (Standard_in, Standard_error, Standard_out)
        } else {
            (Standard_in, Standard_error, Standard_out)
        };

        let Standard_in = self
            .Transfert(
                Standard_in,
                Current_task,
                New_task,
                Some(File_identifier_type::Standard_in),
            )
            .await?;
        let Standard_error = self
            .Transfert(
                Standard_error,
                Current_task,
                New_task,
                Some(File_identifier_type::Standard_error),
            )
            .await?;
        let Standard_out = self
            .Transfert(
                Standard_out,
                Current_task,
                New_task,
                Some(File_identifier_type::Standard_out),
            )
            .await?;

        Ok((Standard_in, Standard_error, Standard_out))
    }

    pub async fn Is_a_terminal(
        &self,
        File: Unique_file_identifier_type,
        Task: Task_identifier_type,
    ) -> Result_type<bool> {
        let (File_system, File) = File.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Pipe_file_system => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::Device_file_system => {
                self.Device_file_system.Is_a_terminal(File).await
            }
            _ => Err(Error_type::Unsupported_operation),
        }
    }

    pub async fn Rename(
        &self,
        Old_path: &impl AsRef<Path_type>,
        New_path: &impl AsRef<Path_type>,
    ) -> Result_type<()> {
        let File_systems = self.File_systems.read().await; // Get the file systems

        let (Old_file_system_identifier, Old_file_system, Old_relative_path) =
            Self::Get_file_system_from_path(&File_systems, Old_path)?; // Get the file system identifier and the relative path

        let (New_file_system_identifier, _, New_relative_path) =
            Self::Get_file_system_from_path(&File_systems, New_path)?; // Get the file system identifier and the relative path

        if Old_file_system_identifier != New_file_system_identifier {
            return Err(Error_type::Invalid_path);
        }

        if Old_file_system_identifier == New_file_system_identifier {
            Old_file_system.Rename(Old_relative_path, New_relative_path)
        } else {
            Err(Error_type::Unsupported_operation) // TODO : Add support for moving between file systems
        }
    }

    pub async fn Get_raw_device(&self, Path: &impl AsRef<Path_type>) -> Result_type<Device_type> {
        let File_systems = self.File_systems.read().await;

        let (_, File_system, Relative_path) = Self::Get_file_system_from_path(&File_systems, Path)?; // Get the file system identifier and the relative path

        let Metadata = File_system.Get_metadata_from_path(Relative_path)?;

        if Metadata.Get_type() != Type_type::Block_device
            && Metadata.Get_type() != Type_type::Character_device
        {
            return Err(Error_type::Unsupported_operation);
        }

        if let Some(Inode) = Metadata.Get_inode() {
            self.Device_file_system.Get_raw_device(Inode).await
        } else {
            Err(Error_type::Corrupted)
        }
    }

    pub async fn Get_metadata_from_path(
        &self,
        Path: &impl AsRef<Path_type>,
    ) -> Result_type<Metadata_type> {
        let File_systems = self.File_systems.read().await;

        let (_, File_system, Relative_path) = Self::Get_file_system_from_path(&File_systems, Path)?; // Get the file system identifier and the relative path

        File_system.Get_metadata_from_path(Relative_path)
    }

    pub async fn Send(
        &self,
        Task: Task_identifier_type,
        Socket: Unique_file_identifier_type,
        Data: &[u8],
    ) -> crate::Result_type<()> {
        let (File_system, Socket) = Socket.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Network_socket_file_system => self
                .Network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Send(Socket, Data)?,
            _ => Err(crate::Error_type::Invalid_file_system)?,
        }

        Ok(())
    }

    pub async fn Receive(
        &self,
        Task: Task_identifier_type,
        Socket: Unique_file_identifier_type,
        Data: &mut [u8],
    ) -> crate::Result_type<usize> {
        let (File_system, Socket) = Socket.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Network_socket_file_system => Ok(self
                .Network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Receive(Socket, Data)?),
            _ => Err(crate::Error_type::Invalid_file_system)?,
        }
    }

    pub async fn Send_to(
        &self,
        Task: Task_identifier_type,
        Socket: Unique_file_identifier_type,
        Data: &[u8],
        Address: Socket_address_type,
    ) -> crate::Result_type<()> {
        let (File_system, Socket) = Socket.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Network_socket_file_system => {
                let (IP, Port) = Address
                    .Into_IP_and_port()
                    .ok_or(crate::Error_type::Invalid_parameter)?;

                self.Network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Send_to(Socket, Data, IP, Port)?
            }
            _ => Err(crate::Error_type::Invalid_file_system)?,
        }

        Ok(())
    }

    pub async fn Receive_from(
        &self,
        Task: Task_identifier_type,
        Socket: Unique_file_identifier_type,
        Data: &mut [u8],
    ) -> crate::Result_type<(usize, Socket_address_type)> {
        let (File_system, Socket) = Socket.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Network_socket_file_system => {
                let (Size, IP, Port) = self
                    .Network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Receive_from(Socket, Data)?;

                Ok((Size, Socket_address_type::From_IP_and_port(IP, Port)))
            }
            _ => Err(crate::Error_type::Invalid_file_system)?,
        }
    }

    fn New_file_identifier(
        &self,
        File_system: File_system_identifier_type,
        Task: Task_identifier_type,
    ) -> crate::Result_type<Local_file_identifier_type> {
        let Iterator = Local_file_identifier_type::Get_minimum(Task).into_iter();

        match File_system {
            File_system_identifier_type::Network_socket_file_system => Ok(self
                .Network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Get_new_socket_identifier(Iterator)?
                .ok_or(crate::Error_type::Too_many_open_files)?),
            _ => Err(crate::Error_type::Invalid_file_system)?,
        }
    }

    pub async fn Bind(
        &self,
        Task: Task_identifier_type,
        Address: Socket_address_type,
        Protocol: Protocol_type,
    ) -> crate::Result_type<Unique_file_identifier_type> {
        let File_system = match Address {
            Socket_address_type::IPv4(_, _) | Socket_address_type::IPv6(_, _) => {
                File_system_identifier_type::Network_socket_file_system
            }
            Socket_address_type::Local(_) => {
                todo!()
            }
        };

        let New_socket = self.New_file_identifier(File_system, Task)?;

        match File_system {
            File_system_identifier_type::Network_socket_file_system => {
                let (IP, Port) = if let Some((IP_type, Port)) = Address.Into_IP_and_port() {
                    (IP_type, Port)
                } else {
                    unreachable!()
                };

                self.Network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Bind(IP, Port, Protocol, New_socket)?
            }
            _ => return Err(crate::Error_type::Invalid_file_system),
        }

        let (_, New_socket) = New_socket.Into_unique_file_identifier(File_system);

        Ok(New_socket)
    }

    pub async fn Connect(
        &self,
        Task: Task_identifier_type,
        Address: Socket_address_type,
    ) -> crate::Result_type<Unique_file_identifier_type> {
        let File_system = match Address {
            Socket_address_type::IPv4(_, _) | Socket_address_type::IPv6(_, _) => {
                File_system_identifier_type::Network_socket_file_system
            }
            Socket_address_type::Local(_) => {
                todo!()
            }
        };

        let New_socket = self.New_file_identifier(File_system, Task)?;

        match File_system {
            File_system_identifier_type::Network_socket_file_system => {
                let (IP, Port) = if let Some((IP_type, Port)) = Address.Into_IP_and_port() {
                    (IP_type, Port)
                } else {
                    unreachable!()
                };

                self.Network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Connect(IP, Port, New_socket)?
            }
            _ => return Err(crate::Error_type::Invalid_file_system),
        }

        let (_, New_socket) = New_socket.Into_unique_file_identifier(File_system);

        Ok(New_socket)
    }

    pub async fn Accept(
        &self,
        Task: Task_identifier_type,
        Socket: Unique_file_identifier_type,
    ) -> crate::Result_type<(Unique_file_identifier_type, Option<(IP_type, Port_type)>)> {
        let (File_system, Socket) = Socket.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Network_socket_file_system => {
                let New_socket = self.New_file_identifier(File_system, Task)?;

                let (IP, Port) = self
                    .Network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Accept(Socket, New_socket)?;

                let (_, New_socket) = New_socket.Into_unique_file_identifier(File_system);

                Ok((New_socket, Some((IP, Port))))
            }
            _ => Err(crate::Error_type::Invalid_file_system),
        }
    }

    pub async fn Set_send_timeout(
        &self,
        Task: Task_identifier_type,
        Socket: Unique_file_identifier_type,
        Timeout: Duration_type,
    ) -> crate::Result_type<()> {
        let (File_system, Socket) = Socket.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Network_socket_file_system => self
                .Network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Set_send_timeout(Socket, Timeout)?,
            _ => return Err(crate::Error_type::Invalid_file_system),
        }

        Ok(())
    }

    pub async fn Set_receive_timeout(
        &self,
        Task: Task_identifier_type,
        Socket: Unique_file_identifier_type,
        Timeout: Duration_type,
    ) -> crate::Result_type<()> {
        let (File_system, Socket) = Socket.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Network_socket_file_system => self
                .Network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Set_receive_timeout(Socket, Timeout)?,
            _ => return Err(crate::Error_type::Invalid_file_system),
        }

        Ok(())
    }

    pub async fn Get_send_timeout(
        &self,
        Task: Task_identifier_type,
        Socket: Unique_file_identifier_type,
    ) -> crate::Result_type<Option<Duration_type>> {
        let (File_system, Socket) = Socket.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Network_socket_file_system => Ok(self
                .Network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Get_send_timeout(Socket)?),
            _ => Err(crate::Error_type::Invalid_file_system),
        }
    }

    pub async fn Get_receive_timeout(
        &self,
        Task: Task_identifier_type,
        Socket: Unique_file_identifier_type,
    ) -> crate::Result_type<Option<Duration_type>> {
        let (File_system, Socket) = Socket.Into_local_file_identifier(Task);

        match File_system {
            File_system_identifier_type::Network_socket_file_system => Ok(self
                .Network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Get_receive_timeout(Socket)?),
            _ => Err(crate::Error_type::Invalid_file_system),
        }
    }
}

#[cfg(test)]
mod Tests {
    use File_system::Local_file_identifier_type;

    use super::*;

    struct Dummy_file_system_type;

    impl File_system_traits for Dummy_file_system_type {
        fn Open(
            &self,
            _: Task_identifier_type,
            _: &Path_type,
            _: Flags_type,
            _: Time_type,
            _: User_identifier_type,
            _: Group_identifier_type,
        ) -> Result_type<Local_file_identifier_type> {
            todo!()
        }

        fn Close(&self, _: Local_file_identifier_type) -> Result_type<()> {
            todo!()
        }

        fn Close_all(&self, _: Task_identifier_type) -> Result_type<()> {
            todo!()
        }

        fn Duplicate(
            &self,
            _: Local_file_identifier_type,
        ) -> Result_type<Local_file_identifier_type> {
            todo!()
        }

        fn Transfert(
            &self,
            _: Task_identifier_type,
            _: Local_file_identifier_type,
            _: Option<File_identifier_type>,
        ) -> Result_type<Local_file_identifier_type> {
            todo!()
        }

        fn Remove(&self, _: &Path_type) -> Result_type<()> {
            todo!()
        }

        fn Read(
            &self,
            _: Local_file_identifier_type,
            _: &mut [u8],
            _: Time_type,
        ) -> Result_type<Size_type> {
            todo!()
        }

        fn Write(
            &self,
            _: Local_file_identifier_type,
            _: &[u8],
            _: Time_type,
        ) -> Result_type<Size_type> {
            todo!()
        }

        fn Rename(&self, _: &Path_type, _: &Path_type) -> Result_type<()> {
            todo!()
        }

        fn Set_position(
            &self,
            _: Local_file_identifier_type,
            _: &Position_type,
        ) -> Result_type<Size_type> {
            todo!()
        }

        fn Flush(&self, _: Local_file_identifier_type) -> Result_type<()> {
            todo!()
        }

        fn Create_directory(
            &self,
            _: &Path_type,
            _: Time_type,
            _: User_identifier_type,
            _: Group_identifier_type,
        ) -> Result_type<()> {
            todo!()
        }

        fn Open_directory(
            &self,
            _: &Path_type,
            _: Task_identifier_type,
        ) -> Result_type<Local_file_identifier_type> {
            todo!()
        }

        fn Read_directory(&self, _: Local_file_identifier_type) -> Result_type<Option<Entry_type>> {
            todo!()
        }

        fn Set_position_directory(
            &self,
            _: Local_file_identifier_type,
            _: Size_type,
        ) -> Result_type<()> {
            todo!()
        }

        fn Get_position_directory(&self, _: Local_file_identifier_type) -> Result_type<Size_type> {
            todo!()
        }

        fn Rewind_directory(&self, _: Local_file_identifier_type) -> Result_type<()> {
            todo!()
        }

        fn Close_directory(&self, _: Local_file_identifier_type) -> Result_type<()> {
            todo!()
        }

        fn Set_metadata_from_path(&self, _: &Path_type, _: &Metadata_type) -> Result_type<()> {
            todo!()
        }

        fn Get_metadata_from_path(&self, _: &Path_type) -> Result_type<Metadata_type> {
            todo!()
        }

        fn Get_statistics(&self, _: Local_file_identifier_type) -> Result_type<Statistics_type> {
            todo!()
        }

        fn Get_mode(&self, _: Local_file_identifier_type) -> Result_type<Mode_type> {
            todo!()
        }

        fn Get_metadata(&self, _: Local_file_identifier_type) -> Result_type<Metadata_type> {
            todo!()
        }
    }

    #[test]
    fn Test_get_file_system_from_path() {
        let mut File_systems: BTreeMap<File_system_identifier_type, Internal_file_system_type> =
            BTreeMap::new();

        File_systems.insert(
            1.into(),
            Internal_file_system_type {
                Mount_point: Path_owned_type::New("/".to_string()).unwrap(),
                Inner: Box::new(Dummy_file_system_type),
            },
        );

        File_systems.insert(
            2.into(),
            Internal_file_system_type {
                Mount_point: Path_owned_type::New("/Foo".to_string()).unwrap(),
                Inner: Box::new(Dummy_file_system_type),
            },
        );

        File_systems.insert(
            3.into(),
            Internal_file_system_type {
                Mount_point: Path_owned_type::New("/Foo/Bar".to_string()).unwrap(),
                Inner: Box::new(Dummy_file_system_type),
            },
        );

        let (File_system, _, Relative_path) =
            Virtual_file_system_type::Get_file_system_from_path(&File_systems, &"/").unwrap();

        assert_eq!(File_system, 1.into());
        assert_eq!(Relative_path, Path_type::Root);

        let (File_system, _, Relative_path) =
            Virtual_file_system_type::Get_file_system_from_path(&File_systems, &"/Foo/Bar")
                .unwrap();

        assert_eq!(File_system, 3.into());
        assert_eq!(Relative_path, Path_type::Root);

        let (File_system, _, Relative_path) =
            Virtual_file_system_type::Get_file_system_from_path(&File_systems, &"/Foo/Bar/Baz")
                .unwrap();

        assert_eq!(File_system, 3.into());
        assert_eq!(Relative_path, "/Baz".as_ref());

        let (File_system, _, Relative_path) =
            Virtual_file_system_type::Get_file_system_from_path(&File_systems, &"/Foo").unwrap();

        assert_eq!(File_system, 2.into());
        assert_eq!(Relative_path, Path_type::Root);
    }
}
