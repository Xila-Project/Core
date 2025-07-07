use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use Futures::yield_now;
use Synchronization::{
    blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, rwlock::RwLock,
};

use alloc::{boxed::Box, collections::BTreeMap};

use Network::{IP_type, Network_socket_driver_trait, Port_type, Protocol_type};
use Task::Task_identifier_type;
use Time::Duration_type;
use Users::{Group_identifier_type, User_identifier_type};

use File_system::{
    Device_type, Entry_type, File_identifier_type, Inode_type, Local_file_identifier_type,
    Metadata_type, Mode_type, Open_type, Statistics_type, Time_type, Type_type,
};

use File_system::{
    Error_type, File_system_identifier_type, File_system_traits, Flags_type, Path_owned_type,
    Path_type, Permissions_type, Position_type, Result_type, Size_type, Status_type,
    Unique_file_identifier_type,
};

use crate::Device::Internal_path_type;
use crate::{Device, Pipe, Socket_address_type};

struct Internal_file_system_type {
    pub mount_point: Path_owned_type,
    pub inner: Box<dyn File_system_traits>,
}

/// Instance of the virtual file system.
///
/// # Safety
/// I know, it is not safe to use mutable static variables.
/// It is thread safe (after initialization) because it is only read after initialization.
/// It is a pragmatic choice for efficiency in embedded systems contexts (avoid using Arc).
static VIRTUAL_FILE_SYSTEM_INSTANCE: OnceLock<Virtual_file_system_type> = OnceLock::new();

pub fn Initialize(
    root_file_system: Box<dyn File_system_traits>,
    network_socket_driver: Option<&'static dyn Network_socket_driver_trait>,
) -> Result<&'static Virtual_file_system_type<'static>, crate::Error_type> {
    let virtual_file_system = Virtual_file_system_type::New(
        Task::Get_instance(),
        Users::Get_instance(),
        Time::Get_instance(),
        root_file_system,
        network_socket_driver,
    )?;

    Ok(VIRTUAL_FILE_SYSTEM_INSTANCE.get_or_init(|| virtual_file_system))
}

pub fn Get_instance() -> &'static Virtual_file_system_type<'static> {
    VIRTUAL_FILE_SYSTEM_INSTANCE
        .try_get()
        .expect("Virtual file system not initialized")
}

/// The virtual file system.
///
/// It is a singleton.
pub struct Virtual_file_system_type<'a> {
    /// Mounted file systems.
    file_systems: RwLock<
        CriticalSectionRawMutex,
        BTreeMap<File_system_identifier_type, Internal_file_system_type>,
    >,
    /// Devices.
    device_file_system: Device::File_system_type<'a>,
    /// Pipes.
    pipe_file_system: Pipe::File_system_type,
    /// Network sockets.
    network_socket_driver: Option<&'a dyn Network_socket_driver_trait>,
}

impl<'a> Virtual_file_system_type<'a> {
    pub const STANDARD_INPUT_FILE_IDENTIFIER: File_identifier_type = File_identifier_type::New(0);
    pub const STANDARD_OUTPUT_FILE_IDENTIFIER: File_identifier_type = File_identifier_type::New(1);
    pub const STANDARD_ERROR_FILE_IDENTIFIER: File_identifier_type = File_identifier_type::New(2);

    pub fn New(
        _: &'static Task::Manager_type,
        _: &'static Users::Manager_type,
        _: &'static Time::Manager_type,
        root_file_system: Box<dyn File_system_traits>,
        network_socket_driver: Option<&'a dyn Network_socket_driver_trait>,
    ) -> Result_type<Self> {
        let mut file_systems = BTreeMap::new();

        let Identifier = Self::Get_new_file_system_identifier(&file_systems)
            .ok_or(Error_type::Too_many_mounted_file_systems)?;

        file_systems.insert(
            Identifier,
            Internal_file_system_type {
                mount_point: Path_owned_type::New("/".to_string()).unwrap(),
                inner: root_file_system,
            },
        );

        Ok(Self {
            file_systems: RwLock::new(file_systems),
            device_file_system: Device::File_system_type::New(),
            pipe_file_system: Pipe::File_system_type::new(),
            network_socket_driver,
        })
    }

    pub async fn Uninitialize(&self) {
        if let Ok(inodes) = self
            .device_file_system
            .Get_devices_from_path(Path_type::ROOT)
            .await
        {
            for Inode in inodes {
                if let Ok(path) = self.device_file_system.Get_path_from_inode(Inode).await {
                    match path {
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
        file_systems: &BTreeMap<File_system_identifier_type, Internal_file_system_type>,
    ) -> Option<File_system_identifier_type> {
        let mut file_system_identifier = File_system_identifier_type::MINIMUM;

        while file_systems.contains_key(&file_system_identifier) {
            file_system_identifier += 1;
        }

        Some(file_system_identifier)
    }

    fn Get_file_system_from_identifier(
        file_systems: &BTreeMap<File_system_identifier_type, Internal_file_system_type>,
        file_system_identifier: File_system_identifier_type,
    ) -> Result_type<&Internal_file_system_type> {
        file_systems
            .get(&file_system_identifier)
            .ok_or(Error_type::Invalid_identifier)
    }

    /// Mount a file system at a given mount point.
    pub async fn Mount_file_system(
        &self,
        file_system: Box<dyn File_system_traits>,
        path: impl AsRef<Path_type>,
        task: Task_identifier_type,
    ) -> Result_type<File_system_identifier_type> {
        if !path.as_ref().Is_valid() {
            return Err(Error_type::Invalid_path);
        }

        let Path = path.as_ref();

        if !Path.Is_absolute() {
            return Err(Error_type::Invalid_path);
        }

        let mut File_systems = self.file_systems.write().await; // Get the file systems

        // Create a directory in the underlying file system
        let (_, Parent_file_system, Relative_path) =
            Self::Get_file_system_from_path(&File_systems, &Path)?; // Get the file system identifier and the relative path

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        Parent_file_system.Create_directory(Relative_path, Time, User, Group)?;

        // Create a directory at the mount point
        let File_system_identifier = Self::Get_new_file_system_identifier(&File_systems)
            .ok_or(Error_type::Too_many_mounted_file_systems)?;

        File_systems.insert(
            File_system_identifier,
            Internal_file_system_type {
                mount_point: Path.to_owned(),
                inner: file_system,
            },
        );

        Ok(File_system_identifier)
    }

    pub async fn Unmount_file_system(
        &self,
        path: impl AsRef<Path_type>,
        task: Task_identifier_type,
    ) -> Result_type<()> {
        let path = path.as_ref();

        if !path.Is_valid() || !path.Is_absolute() {
            return Err(Error_type::Invalid_path);
        }

        let mut File_systems = self.file_systems.write().await; // Get the file systems

        let File_system_identifier = {
            let (file_system_identifier, _, Relative_path) =
                Self::Get_file_system_from_path(&File_systems, &path)?; // Get the file system identifier and the relative path

            if !Relative_path.Is_root() {
                return Err(Error_type::Invalid_path);
            }

            file_system_identifier
        };

        let File_system = File_systems
            .remove(&File_system_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        File_system.inner.Close_all(task)?;

        let (_, Parent_file_system, Relative_path) =
            Self::Get_file_system_from_path(&File_systems, &File_system.mount_point)?;

        Parent_file_system.Remove(Relative_path)?;

        Ok(())
    }

    fn Get_file_system_from_path<'b>(
        file_systems: &'b BTreeMap<File_system_identifier_type, Internal_file_system_type>,
        path: &'b impl AsRef<Path_type>,
    ) -> Result_type<(
        File_system_identifier_type,
        &'b dyn File_system_traits,
        &'b Path_type,
    )> {
        let mut Result_score = 0;
        let mut result: Option<File_system_identifier_type> = None;

        let Path = path.as_ref();
        let path_components = Path.Get_components();

        for (File_system_identifier, File_system) in file_systems.iter() {
            let mount_point: &Path_type = File_system.mount_point.as_ref();
            let mount_point_components = mount_point.Get_components();

            let Score = path_components
                .clone()
                .Get_common_components(mount_point_components);

            if Result_score < Score {
                Result_score = Score;
                result = Some(*File_system_identifier);
            }
        }

        let File_system_identifier = result.ok_or(Error_type::Invalid_path)?;

        let File_system = file_systems
            .get(&File_system_identifier)
            .ok_or(Error_type::Invalid_path)?;

        let Relative_path = Path
            .Strip_prefix_absolute(File_system.mount_point.as_ref())
            .ok_or(Error_type::Invalid_path)?;

        Ok((
            File_system_identifier,
            File_system.inner.as_ref(),
            Relative_path,
        ))
    }

    pub async fn Open(
        &self,
        path: &impl AsRef<Path_type>,
        flags: Flags_type,
        task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (File_system_identifier, File_system, Relative_path) =
            Self::Get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let Time: Time_type = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        let Local_file = File_system.Open(task, Relative_path, flags, Time, User, Group)?;

        let Metadata = File_system.Get_metadata(Local_file)?;

        let (_, Unique_file) = Local_file.Into_unique_file_identifier(File_system_identifier);

        let Unique_file = match Metadata.Get_type() {
            Type_type::Character_device | Type_type::Block_device => {
                if let Some(inode) = Metadata.Get_inode() {
                    let local_file = self
                        .device_file_system
                        .Open(inode, task, flags, Unique_file)
                        .await?;

                    local_file
                        .Into_unique_file_identifier(
                            File_system_identifier_type::DEVICE_FILE_SYSTEM,
                        )
                        .1
                } else {
                    return Err(Error_type::Corrupted)?;
                }
            }
            Type_type::Pipe => {
                if let Some(inode) = Metadata.Get_inode() {
                    let local_file = self
                        .pipe_file_system
                        .Open(inode, task, flags, Unique_file)
                        .await?;

                    local_file
                        .Into_unique_file_identifier(File_system_identifier_type::PIPE_FILE_SYSTEM)
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
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
    ) -> crate::Result_type<()> {
        let (file_system, local_file) = file.Into_local_file_identifier(task);

        let Underlying_file = match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                match self.pipe_file_system.Close(local_file).await? {
                    Some(underlying_file) => underlying_file,
                    None => return Ok(()),
                }
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                self.device_file_system.Close(local_file).await?
            }
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => {
                self.network_socket_driver
                    .ok_or(Error_type::Unsupported_operation)?
                    .Close(local_file)?;

                return Ok(());
            }
            _ => {
                self.file_systems
                    .read()
                    .await
                    .get(&file_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .inner
                    .Close(local_file)?;

                return Ok(());
            }
        };

        // - If there is an underlying file (some pipe and devices), close it too.
        let (File_system, Local_file) = Underlying_file.Into_local_file_identifier(task);

        self.file_systems
            .read()
            .await
            .get(&File_system)
            .ok_or(Error_type::Invalid_identifier)?
            .inner
            .Close(Local_file)?;

        Ok(())
    }

    pub async fn Read(
        &self,
        file: Unique_file_identifier_type,
        buffer: &mut [u8],
        task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (file_system, local_file_identifier) = file.Into_local_file_identifier(task);

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let (Size, Underlying_file) = match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system
                    .Read(local_file_identifier, buffer)
                    .await?
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                let result = self
                    .device_file_system
                    .Read(local_file_identifier, buffer)
                    .await?;
                (result.0, Some(result.1))
            }
            _ => {
                return self
                    .file_systems
                    .read()
                    .await
                    .get(&file_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .inner
                    .Read(local_file_identifier, buffer, Time)
            }
        };

        if let Some(Underlying_file) = Underlying_file {
            let (file_system, local_file_identifier) =
                Underlying_file.Into_local_file_identifier(task);

            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Read(local_file_identifier, &mut [0; 0], Time)?;
        }
        Ok(Size)
    }

    pub async fn Read_line(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
        buffer: &mut String,
    ) -> Result_type<Size_type> {
        let (file_system, local_file_identifier) = file.Into_local_file_identifier(task);

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let (Size, Underlying_file) = match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system
                    .Read_line(local_file_identifier, buffer)
                    .await?
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                let result = self
                    .device_file_system
                    .Read_line(local_file_identifier, buffer)
                    .await?;
                (result.0, Some(result.1))
            }
            _ => {
                let File_systems = self.file_systems.read().await; // Get the file systems

                let File_system = &File_systems
                    .get(&file_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .inner;

                return Read_line(&**File_system, buffer, local_file_identifier, Time).await;
            }
        };

        if let Some(Underlying_file) = Underlying_file {
            let (file_system, local_file_identifier) =
                Underlying_file.Into_local_file_identifier(task);

            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Read(local_file_identifier, &mut [0; 0], Time)?;
        }

        Ok(Size)
    }

    pub async fn Read_to_end(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
        buffer: &mut Vec<u8>,
    ) -> Result_type<Size_type> {
        const CHUNK_SIZE: usize = 512;

        let mut Read_size = 0;

        loop {
            let mut Chunk = vec![0; CHUNK_SIZE];

            let Size: usize = self.Read(file, &mut Chunk, task).await?.into();

            if Size == 0 {
                break;
            }

            buffer.extend_from_slice(&Chunk[..Size]);

            Read_size += Size;
        }

        Ok(Read_size.into())
    }

    pub async fn Write(
        &self,
        file: Unique_file_identifier_type,
        buffer: &[u8],
        task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (file_system, local_file_identifier) = file.Into_local_file_identifier(task);

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let (Size, Underlying_file) = match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system
                    .Write(local_file_identifier, buffer)
                    .await?
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                let result = self
                    .device_file_system
                    .Write(local_file_identifier, buffer)
                    .await?;
                (result.0, Some(result.1))
            }
            _ => {
                return self
                    .file_systems
                    .read()
                    .await
                    .get(&file_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .inner
                    .Write(local_file_identifier, buffer, Time)
            }
        };

        if let Some(Underlying_file) = Underlying_file {
            let (file_system, local_file_identifier) =
                Underlying_file.Into_local_file_identifier(task);

            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Write(local_file_identifier, &[0; 0], Time)?;
        }

        Ok(Size)
    }

    pub async fn Set_position(
        &self,
        file: Unique_file_identifier_type,
        position: &Position_type,
        task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (file_system, local_file) = file.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                let (size, underlying_file) = self
                    .device_file_system
                    .Set_position(local_file, position)
                    .await?;

                let (File_system, Local_file) = underlying_file.Into_local_file_identifier(task);

                self.file_systems
                    .read()
                    .await
                    .get(&File_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .inner
                    .Set_position(Local_file, position)?;

                Ok(size)
            }
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Set_position(local_file, position),
        }
    }

    pub async fn Set_owner(
        &self,
        path: impl AsRef<Path_type>,
        user: Option<User_identifier_type>,
        group: Option<Group_identifier_type>,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let mut Metadata = File_system.Get_metadata_from_path(Relative_path)?;

        if let Some(User) = user {
            Metadata.Set_owner(User);
        }

        if let Some(Group) = group {
            Metadata.Set_group(Group);
        }

        File_system.Set_metadata_from_path(Relative_path, &Metadata)
    }

    pub async fn Set_permissions(
        &self,
        path: impl AsRef<Path_type>,
        permissions: Permissions_type,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let mut Metadata = File_system.Get_metadata_from_path(Relative_path)?;

        Metadata.Set_permissions(permissions);

        File_system.Set_metadata_from_path(Relative_path, &Metadata)
    }

    pub async fn Close_all(&self, Task_identifier: Task_identifier_type) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        for File_system in file_systems.values() {
            File_system.inner.Close_all(Task_identifier)?;
        }

        self.device_file_system.Close_all(Task_identifier).await?;

        self.pipe_file_system.Close_all(Task_identifier).await?;

        Ok(())
    }

    pub async fn Mount_device(
        &self,
        task: Task_identifier_type,
        path: &impl AsRef<Path_type>,
        device: Device_type,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        let File = File_system.Open(
            task,
            Relative_path,
            Flags_type::New(Mode_type::READ_WRITE, Some(Open_type::CREATE_ONLY), None),
            Time,
            User,
            Group,
        )?;

        File_system.Close(File)?;

        let Inode = self
            .device_file_system
            .Mount_device(Relative_path.to_owned(), device)
            .await?;

        let Time: Time_type = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        let mut Metadata = Metadata_type::Get_default(Type_type::Block_device, Time, User, Group)
            .ok_or(Error_type::Invalid_parameter)?;
        Metadata.Set_inode(Inode);

        File_system.Set_metadata_from_path(Relative_path, &Metadata)?;

        Ok(())
    }

    pub async fn Mount_static_device(
        &self,
        task: Task_identifier_type,
        path: &'a impl AsRef<Path_type>,
        device: Device_type,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        // Create a special file in the underlying file system.
        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        let File = File_system.Open(
            task,
            Relative_path,
            Flags_type::New(Mode_type::WRITE_ONLY, Some(Open_type::CREATE_ONLY), None),
            Time,
            User,
            Group,
        )?;

        File_system.Close(File)?;

        let Type = if device.Is_a_block_device() {
            Type_type::Block_device
        } else {
            Type_type::Character_device
        };

        // Create the actual device.
        let Inode = self
            .device_file_system
            .Mount_static_device(path, device)
            .await?;

        let Time: Time_type = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(task).await?;

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
        path: &impl AsRef<Path_type>,
        size: usize,
        task: Task_identifier_type,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, File_system, Relative_path) = Self::Get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        let File = File_system.Open(
            task,
            Relative_path,
            Flags_type::New(Mode_type::READ_WRITE, Some(Open_type::CREATE_ONLY), None),
            Time,
            User,
            Group,
        )?;

        File_system.Close(File)?;

        let Inode = self.pipe_file_system.Create_named_pipe(size).await?;

        let Time: Time_type = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        let mut Metadata = Metadata_type::Get_default(Type_type::Pipe, Time, User, Group)
            .ok_or(Error_type::Invalid_parameter)?;
        Metadata.Set_inode(Inode);

        File_system.Set_metadata_from_path(Relative_path, &Metadata)?;

        Ok(())
    }

    pub async fn Create_unnamed_pipe(
        &self,
        task: Task_identifier_type,
        status: Status_type,
        size: usize,
    ) -> Result_type<(Unique_file_identifier_type, Unique_file_identifier_type)> {
        let (read, write) = self
            .pipe_file_system
            .Create_unnamed_pipe(task, status, size)
            .await?;

        let (_, Read) =
            read.Into_unique_file_identifier(File_system_identifier_type::PIPE_FILE_SYSTEM);
        let (_, write) =
            write.Into_unique_file_identifier(File_system_identifier_type::PIPE_FILE_SYSTEM);

        Ok((Read, write))
    }

    pub async fn Remove(&self, Path: impl AsRef<Path_type>) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        // - Check metadata on the underlying file system
        let (_, File_system, Relative_path) =
            Self::Get_file_system_from_path(&file_systems, &Path)?; // Get the file system identifier and the relative path

        let Metadata = File_system.Get_metadata_from_path(Relative_path)?;

        File_system.Remove(Relative_path)?;

        match Metadata.Get_type() {
            Type_type::Pipe => {
                if let Some(inode) = Metadata.Get_inode() {
                    match self.pipe_file_system.Remove(inode).await {
                        Ok(_) | Err(Error_type::Invalid_inode) => (),
                        Err(error) => {
                            return Err(error);
                        }
                    }
                }
            }
            Type_type::Block_device => {
                if let Some(inode) = Metadata.Get_inode() {
                    match self.device_file_system.Remove(inode).await {
                        Ok(_) | Err(Error_type::Invalid_inode) => (),
                        Err(error) => {
                            return Err(error);
                        }
                    }
                }
            }
            Type_type::Character_device => {
                if let Some(inode) = Metadata.Get_inode() {
                    match self.device_file_system.Remove(inode).await {
                        Ok(_) | Err(Error_type::Invalid_inode) => (),
                        Err(error) => {
                            return Err(error);
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
        file: Unique_file_identifier_type,
        current_task: Task_identifier_type,
        new_task: Task_identifier_type,
        new_file: Option<File_identifier_type>,
    ) -> Result_type<Unique_file_identifier_type> {
        let (file_system, file) = file.Into_local_file_identifier(current_task);

        let Underlying_file = match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system.Get_underlying_file(file).await?
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                Some(self.device_file_system.Get_underlying_file(file).await?)
            }
            _ => None,
        };

        let File_systems = self.file_systems.read().await;

        let Underlying_file = if let Some(Underlying_file) = Underlying_file {
            let (file_system, local_file) =
                Underlying_file.Into_local_file_identifier(current_task);

            Some(
                File_systems
                    .get(&file_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .inner
                    .Transfert(new_task, local_file, new_file)?
                    .Into_unique_file_identifier(file_system)
                    .1,
            )
        } else {
            None
        };

        let New_file = match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system
                    .Transfert(new_task, file, new_file)
                    .await?
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                let underlying_file = Underlying_file.ok_or(Error_type::Internal_error)?;

                self.device_file_system
                    .Transfert(new_task, file, underlying_file, new_file)
                    .await?
            }
            _ => Self::Get_file_system_from_identifier(&File_systems, file_system)?
                .inner
                .Transfert(new_task, file, new_file)?,
        };

        let (_, New_file) = New_file.Into_unique_file_identifier(file_system);

        Ok(New_file)
    }

    pub async fn Flush(
        &self,
        file: Unique_file_identifier_type,
        task_identifier: Task_identifier_type,
    ) -> Result_type<()> {
        let (file_system, file_identifier) = file.Into_local_file_identifier(task_identifier);

        if file_system == File_system_identifier_type::PIPE_FILE_SYSTEM {
            Ok(())
        } else if file_system == File_system_identifier_type::DEVICE_FILE_SYSTEM {
            let underlying_file = self.device_file_system.Flush(file_identifier).await?;

            let (File_system, Local_file) =
                underlying_file.Into_local_file_identifier(task_identifier);

            self.file_systems
                .read()
                .await
                .get(&File_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Flush(Local_file)?;

            Ok(())
        } else {
            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Flush(file_identifier)
        }
    }

    pub async fn Get_statistics(
        &self,
        file: Unique_file_identifier_type,
        task_identifier: Task_identifier_type,
    ) -> Result_type<Statistics_type> {
        let (file_system, local_file) = file.Into_local_file_identifier(task_identifier);

        let File_systems = self.file_systems.read().await;

        let File = match file_system {
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                self.device_file_system
                    .Get_underlying_file(local_file)
                    .await?
            }
            File_system_identifier_type::PIPE_FILE_SYSTEM => self
                .pipe_file_system
                .Get_underlying_file(local_file)
                .await?
                .ok_or(Error_type::Unsupported_operation)?,
            _ => file,
        };

        let (File_system, Local_file) = File.Into_local_file_identifier(task_identifier);

        Self::Get_file_system_from_identifier(&File_systems, File_system)?
            .inner
            .Get_statistics(Local_file)
    }

    pub async fn Open_directory(
        &self,
        path: &impl AsRef<Path_type>,
        task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (File_system_identifier, File_system, Relative_path) =
            Self::Get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let (_, File) = File_system
            .Open_directory(Relative_path, task)?
            .Into_unique_file_identifier(File_system_identifier);

        Ok(File)
    }

    pub async fn Read_directory(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
    ) -> Result_type<Option<Entry_type>> {
        let (file_system, file) = file.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                Err(Error_type::Unsupported_operation)
            }
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Read_directory(file),
        }
    }

    pub async fn Set_position_directory(
        &self,
        file: Unique_file_identifier_type,
        position: Size_type,
        task: Task_identifier_type,
    ) -> Result_type<()> {
        let (file_system, file) = file.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                Err(Error_type::Unsupported_operation)
            }
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Set_position_directory(file, position),
        }
    }

    pub async fn Get_position_directory(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (file_system, file) = file.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                Err(Error_type::Unsupported_operation)
            }
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Get_position_directory(file),
        }
    }

    pub async fn Rewind_directory(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
    ) -> Result_type<()> {
        let (file_system, file) = file.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                Err(Error_type::Unsupported_operation)
            }
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Rewind_directory(file),
        }
    }

    pub async fn Close_directory(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
    ) -> Result_type<()> {
        let (file_system, file) = file.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                Err(Error_type::Unsupported_operation)
            }
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Close_directory(file),
        }
    }

    pub async fn Create_directory(
        &self,
        path: &impl AsRef<Path_type>,
        task: Task_identifier_type,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, File_system, Relative_path) = Self::Get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let Time = Time::Get_instance()
            .Get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let User = Task::Get_instance().Get_user(task).await?;

        let Group = Users::Get_instance().Get_user_primary_group(User).await?;

        File_system.Create_directory(Relative_path, Time, User, Group)
    }

    pub async fn Get_mode(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
    ) -> Result_type<Mode_type> {
        let (file_system, file) = file.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system.Get_mode(file).await
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                self.device_file_system.Get_mode(file).await
            }
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Get_mode(file),
        }
    }

    pub async fn Duplicate_file_identifier(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let (file_system, file) = file.Into_local_file_identifier(task);

        let Underlying_file = match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system.Get_underlying_file(file).await?
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                Some(self.device_file_system.Get_underlying_file(file).await?)
            }
            _ => None,
        };

        let File_systems = self.file_systems.read().await;

        let Underlying_file = if let Some(Underlying_file) = Underlying_file {
            let (file_system, local_file) = Underlying_file.Into_local_file_identifier(task);

            Some(
                File_systems
                    .get(&file_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .inner
                    .Duplicate(local_file)?
                    .Into_unique_file_identifier(file_system)
                    .1,
            )
        } else {
            None
        };

        let New_file = match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system
                    .Duplicate(file, Underlying_file)
                    .await?
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                let underlying_file = Underlying_file.ok_or(Error_type::Internal_error)?;

                self.device_file_system
                    .Duplicate(file, underlying_file)
                    .await?
            }
            _ => Self::Get_file_system_from_identifier(&File_systems, file_system)?
                .inner
                .Duplicate(file)?,
        };

        let (_, New_file) = New_file.Into_unique_file_identifier(file_system);

        Ok(New_file)
    }

    pub async fn Create_new_task_standard_io(
        &self,
        standard_in: Unique_file_identifier_type,
        standard_error: Unique_file_identifier_type,
        standard_out: Unique_file_identifier_type,
        current_task: Task_identifier_type,
        new_task: Task_identifier_type,
        duplicate: bool,
    ) -> Result_type<(
        Unique_file_identifier_type,
        Unique_file_identifier_type,
        Unique_file_identifier_type,
    )> {
        let (Standard_in, Standard_error, Standard_out) = if duplicate {
            let standard_in = self
                .Duplicate_file_identifier(standard_in, current_task)
                .await?;
            let standard_error = self
                .Duplicate_file_identifier(standard_error, current_task)
                .await?;
            let standard_out = self
                .Duplicate_file_identifier(standard_out, current_task)
                .await?;

            (standard_in, standard_error, standard_out)
        } else {
            (standard_in, standard_error, standard_out)
        };

        let Standard_in = self
            .Transfert(
                Standard_in,
                current_task,
                new_task,
                Some(File_identifier_type::STANDARD_IN),
            )
            .await?;
        let standard_error = self
            .Transfert(
                Standard_error,
                current_task,
                new_task,
                Some(File_identifier_type::STANDARD_ERROR),
            )
            .await?;
        let standard_out = self
            .Transfert(
                Standard_out,
                current_task,
                new_task,
                Some(File_identifier_type::STANDARD_OUT),
            )
            .await?;

        Ok((Standard_in, standard_error, standard_out))
    }

    pub async fn Is_a_terminal(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
    ) -> Result_type<bool> {
        let (file_system, file) = file.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                self.device_file_system.Is_a_terminal(file).await
            }
            _ => Err(Error_type::Unsupported_operation),
        }
    }

    pub async fn Rename(
        &self,
        old_path: &impl AsRef<Path_type>,
        new_path: &impl AsRef<Path_type>,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (Old_file_system_identifier, Old_file_system, Old_relative_path) =
            Self::Get_file_system_from_path(&file_systems, old_path)?; // Get the file system identifier and the relative path

        let (New_file_system_identifier, _, New_relative_path) =
            Self::Get_file_system_from_path(&file_systems, new_path)?; // Get the file system identifier and the relative path

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
        let file_systems = self.file_systems.read().await;

        let (_, File_system, Relative_path) = Self::Get_file_system_from_path(&file_systems, Path)?; // Get the file system identifier and the relative path

        let Metadata = File_system.Get_metadata_from_path(Relative_path)?;

        if Metadata.Get_type() != Type_type::Block_device
            && Metadata.Get_type() != Type_type::Character_device
        {
            return Err(Error_type::Unsupported_operation);
        }

        if let Some(Inode) = Metadata.Get_inode() {
            self.device_file_system.Get_raw_device(Inode).await
        } else {
            Err(Error_type::Corrupted)
        }
    }

    pub async fn Get_metadata_from_path(
        &self,
        path: &impl AsRef<Path_type>,
    ) -> Result_type<Metadata_type> {
        let file_systems = self.file_systems.read().await;

        let (_, File_system, Relative_path) = Self::Get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        File_system.Get_metadata_from_path(Relative_path)
    }

    pub async fn Get_statistics_from_path(
        &self,
        path: &impl AsRef<Path_type>,
    ) -> Result_type<Statistics_type> {
        let file_systems = self.file_systems.read().await;

        let (File_system_identifier, File_system, Relative_path) =
            Self::Get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let Metadata = File_system.Get_metadata_from_path(Relative_path)?;

        Ok(Statistics_type::new(
            File_system_identifier,
            Metadata.Get_inode().unwrap_or(Inode_type::New(0)),
            0,
            Size_type::New(0),
            Metadata.Get_access_time(),
            Metadata.Get_modification_time(),
            Metadata.Get_creation_time(),
            Metadata.Get_type(),
            Metadata.Get_permissions(),
            Metadata.Get_user(),
            Metadata.Get_group(),
        ))
    }

    pub async fn Send(
        &self,
        task: Task_identifier_type,
        socket: Unique_file_identifier_type,
        data: &[u8],
    ) -> crate::Result_type<()> {
        let (file_system, socket) = socket.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => self
                .network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Send(socket, data)?,
            _ => Err(crate::Error_type::Invalid_file_system)?,
        }

        Ok(())
    }

    pub async fn Receive(
        &self,
        task: Task_identifier_type,
        socket: Unique_file_identifier_type,
        data: &mut [u8],
    ) -> crate::Result_type<usize> {
        let (file_system, socket) = socket.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => Ok(self
                .network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Receive(socket, data)?),
            _ => Err(crate::Error_type::Invalid_file_system)?,
        }
    }

    pub async fn Send_to(
        &self,
        task: Task_identifier_type,
        socket: Unique_file_identifier_type,
        data: &[u8],
        address: Socket_address_type,
    ) -> crate::Result_type<()> {
        let (file_system, socket) = socket.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => {
                let (ip, port) = address
                    .into_ip_and_port()
                    .ok_or(crate::Error_type::Invalid_parameter)?;

                self.network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Send_to(socket, data, ip, port)?
            }
            _ => Err(crate::Error_type::Invalid_file_system)?,
        }

        Ok(())
    }

    pub async fn Receive_from(
        &self,
        task: Task_identifier_type,
        socket: Unique_file_identifier_type,
        data: &mut [u8],
    ) -> crate::Result_type<(usize, Socket_address_type)> {
        let (file_system, socket) = socket.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => {
                let (size, ip, port) = self
                    .network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Receive_from(socket, data)?;

                Ok((size, Socket_address_type::From_IP_and_port(ip, port)))
            }
            _ => Err(crate::Error_type::Invalid_file_system)?,
        }
    }

    fn New_file_identifier(
        &self,
        file_system: File_system_identifier_type,
        task: Task_identifier_type,
    ) -> crate::Result_type<Local_file_identifier_type> {
        let iterator = Local_file_identifier_type::Get_minimum(task).into_iter();

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => Ok(self
                .network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .get_new_socket_identifier(iterator)?
                .ok_or(crate::Error_type::Too_many_open_files)?),
            _ => Err(crate::Error_type::Invalid_file_system)?,
        }
    }

    pub async fn Bind(
        &self,
        task: Task_identifier_type,
        address: Socket_address_type,
        protocol: Protocol_type,
    ) -> crate::Result_type<Unique_file_identifier_type> {
        let file_system = match address {
            Socket_address_type::IPv4(_, _) | Socket_address_type::IPv6(_, _) => {
                File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM
            }
            Socket_address_type::Local(_) => {
                todo!()
            }
        };

        let New_socket = self.New_file_identifier(file_system, task)?;

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => {
                let (ip, port) = if let Some((ip_type, port)) = address.into_ip_and_port() {
                    (ip_type, port)
                } else {
                    unreachable!()
                };

                self.network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Bind(ip, port, protocol, New_socket)?
            }
            _ => return Err(crate::Error_type::Invalid_file_system),
        }

        let (_, New_socket) = New_socket.Into_unique_file_identifier(file_system);

        Ok(New_socket)
    }

    pub async fn Connect(
        &self,
        task: Task_identifier_type,
        address: Socket_address_type,
    ) -> crate::Result_type<Unique_file_identifier_type> {
        let file_system = match address {
            Socket_address_type::IPv4(_, _) | Socket_address_type::IPv6(_, _) => {
                File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM
            }
            Socket_address_type::Local(_) => {
                todo!()
            }
        };

        let New_socket = self.New_file_identifier(file_system, task)?;

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => {
                let (ip, port) = if let Some((ip_type, port)) = address.into_ip_and_port() {
                    (ip_type, port)
                } else {
                    unreachable!()
                };

                self.network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Connect(ip, port, New_socket)?
            }
            _ => return Err(crate::Error_type::Invalid_file_system),
        }

        let (_, New_socket) = New_socket.Into_unique_file_identifier(file_system);

        Ok(New_socket)
    }

    pub async fn Accept(
        &self,
        task: Task_identifier_type,
        socket: Unique_file_identifier_type,
    ) -> crate::Result_type<(Unique_file_identifier_type, Option<(IP_type, Port_type)>)> {
        let (file_system, socket) = socket.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => {
                let new_socket = self.New_file_identifier(file_system, task)?;

                let (IP, Port) = self
                    .network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Accept(socket, new_socket)?;

                let (_, New_socket) = new_socket.Into_unique_file_identifier(file_system);

                Ok((New_socket, Some((IP, Port))))
            }
            _ => Err(crate::Error_type::Invalid_file_system),
        }
    }

    pub async fn Set_send_timeout(
        &self,
        task: Task_identifier_type,
        socket: Unique_file_identifier_type,
        timeout: Duration_type,
    ) -> crate::Result_type<()> {
        let (file_system, socket) = socket.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => self
                .network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Set_send_timeout(socket, timeout)?,
            _ => return Err(crate::Error_type::Invalid_file_system),
        }

        Ok(())
    }

    pub async fn Set_receive_timeout(
        &self,
        task: Task_identifier_type,
        socket: Unique_file_identifier_type,
        timeout: Duration_type,
    ) -> crate::Result_type<()> {
        let (file_system, socket) = socket.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => self
                .network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Set_receive_timeout(socket, timeout)?,
            _ => return Err(crate::Error_type::Invalid_file_system),
        }

        Ok(())
    }

    pub async fn Get_send_timeout(
        &self,
        task: Task_identifier_type,
        socket: Unique_file_identifier_type,
    ) -> crate::Result_type<Option<Duration_type>> {
        let (file_system, socket) = socket.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => Ok(self
                .network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Get_send_timeout(socket)?),
            _ => Err(crate::Error_type::Invalid_file_system),
        }
    }

    pub async fn Get_receive_timeout(
        &self,
        task: Task_identifier_type,
        socket: Unique_file_identifier_type,
    ) -> crate::Result_type<Option<Duration_type>> {
        let (file_system, socket) = socket.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => Ok(self
                .network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .Get_receive_timeout(socket)?),
            _ => Err(crate::Error_type::Invalid_file_system),
        }
    }
}

async fn Read_line(
    file_system: &dyn File_system_traits,
    buffer: &mut String,
    file: Local_file_identifier_type,
    time: Time_type,
) -> Result_type<Size_type> {
    loop {
        let Current_buffer = &mut [0; 1];

        let Size = file_system.Read(file, Current_buffer, time)?;

        if Size == 0 {
            yield_now().await; // Yield to allow other tasks to run, especially in a blocking operation
            continue; // Retry reading if no data was read
        }

        let Byte = Current_buffer[0];

        if Byte == b'\n' || Byte == b'\r' {
            break;
        }

        buffer.push(Byte as char);
    }

    Ok(buffer.len().into())
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
                mount_point: Path_owned_type::New("/".to_string()).unwrap(),
                inner: Box::new(Dummy_file_system_type),
            },
        );

        File_systems.insert(
            2.into(),
            Internal_file_system_type {
                mount_point: Path_owned_type::New("/Foo".to_string()).unwrap(),
                inner: Box::new(Dummy_file_system_type),
            },
        );

        File_systems.insert(
            3.into(),
            Internal_file_system_type {
                mount_point: Path_owned_type::New("/Foo/Bar".to_string()).unwrap(),
                inner: Box::new(Dummy_file_system_type),
            },
        );

        let (File_system, _, Relative_path) =
            Virtual_file_system_type::Get_file_system_from_path(&File_systems, &"/").unwrap();

        assert_eq!(File_system, 1.into());
        assert_eq!(Relative_path, Path_type::ROOT);

        let (File_system, _, Relative_path) =
            Virtual_file_system_type::Get_file_system_from_path(&File_systems, &"/Foo/Bar")
                .unwrap();

        assert_eq!(File_system, 3.into());
        assert_eq!(Relative_path, Path_type::ROOT);

        let (File_system, _, Relative_path) =
            Virtual_file_system_type::Get_file_system_from_path(&File_systems, &"/Foo/Bar/Baz")
                .unwrap();

        assert_eq!(File_system, 3.into());
        assert_eq!(Relative_path, "/Baz".as_ref());

        let (File_system, _, Relative_path) =
            Virtual_file_system_type::Get_file_system_from_path(&File_systems, &"/Foo").unwrap();

        assert_eq!(File_system, 2.into());
        assert_eq!(Relative_path, Path_type::ROOT);
    }
}
