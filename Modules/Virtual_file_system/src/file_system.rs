use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use futures::yield_now;
use synchronization::{
    blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, rwlock::RwLock,
};

use alloc::{boxed::Box, collections::BTreeMap};

use network::{IP_type, Network_socket_driver_trait, Port_type, Protocol_type};
use task::Task_identifier_type;
use time::Duration_type;
use users::{Group_identifier_type, User_identifier_type};

use file_system::{
    Device_type, Entry_type, File_identifier_type, Inode_type, Local_file_identifier_type,
    Metadata_type, Mode_type, Open_type, Statistics_type, Time_type, Type_type,
};

use file_system::{
    Error_type, File_system_identifier_type, File_system_traits, Flags_type, Path_owned_type,
    Path_type, Permissions_type, Position_type, Result_type, Size_type, Status_type,
    Unique_file_identifier_type,
};

use crate::device::Internal_path_type;
use crate::{device, pipe, Socket_address_type};

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

pub fn initialize(
    root_file_system: Box<dyn File_system_traits>,
    network_socket_driver: Option<&'static dyn Network_socket_driver_trait>,
) -> Result<&'static Virtual_file_system_type<'static>, crate::Error_type> {
    let virtual_file_system = Virtual_file_system_type::new(
        task::get_instance(),
        users::get_instance(),
        time::get_instance(),
        root_file_system,
        network_socket_driver,
    )?;

    Ok(VIRTUAL_FILE_SYSTEM_INSTANCE.get_or_init(|| virtual_file_system))
}

pub fn get_instance() -> &'static Virtual_file_system_type<'static> {
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
    device_file_system: device::File_system_type<'a>,
    /// Pipes.
    pipe_file_system: pipe::File_system_type,
    /// Network sockets.
    network_socket_driver: Option<&'a dyn Network_socket_driver_trait>,
}

impl<'a> Virtual_file_system_type<'a> {
    pub const STANDARD_INPUT_FILE_IDENTIFIER: File_identifier_type = File_identifier_type::New(0);
    pub const STANDARD_OUTPUT_FILE_IDENTIFIER: File_identifier_type = File_identifier_type::New(1);
    pub const STANDARD_ERROR_FILE_IDENTIFIER: File_identifier_type = File_identifier_type::New(2);

    pub fn new(
        _: &'static task::Manager_type,
        _: &'static users::Manager_type,
        _: &'static time::Manager_type,
        root_file_system: Box<dyn File_system_traits>,
        network_socket_driver: Option<&'a dyn Network_socket_driver_trait>,
    ) -> Result_type<Self> {
        let mut file_systems = BTreeMap::new();

        let identifier = Self::get_new_file_system_identifier(&file_systems)
            .ok_or(Error_type::Too_many_mounted_file_systems)?;

        file_systems.insert(
            identifier,
            Internal_file_system_type {
                mount_point: Path_owned_type::New("/".to_string()).unwrap(),
                inner: root_file_system,
            },
        );

        Ok(Self {
            file_systems: RwLock::new(file_systems),
            device_file_system: device::File_system_type::New(),
            pipe_file_system: pipe::File_system_type::new(),
            network_socket_driver,
        })
    }

    pub async fn uninitialize(&self) {
        if let Ok(inodes) = self
            .device_file_system
            .get_devices_from_path(Path_type::ROOT)
            .await
        {
            for inode in inodes {
                if let Ok(path) = self.device_file_system.get_path_from_inode(inode).await {
                    match path {
                        Internal_path_type::Owned(path) => {
                            let _ = self.remove(path).await;
                        }
                        Internal_path_type::Borrowed(path) => {
                            let _ = self.remove(path).await;
                        }
                    }
                }
            }
        }
    }

    fn get_new_file_system_identifier(
        file_systems: &BTreeMap<File_system_identifier_type, Internal_file_system_type>,
    ) -> Option<File_system_identifier_type> {
        let mut file_system_identifier = File_system_identifier_type::MINIMUM;

        while file_systems.contains_key(&file_system_identifier) {
            file_system_identifier += 1;
        }

        Some(file_system_identifier)
    }

    fn get_file_system_from_identifier(
        file_systems: &BTreeMap<File_system_identifier_type, Internal_file_system_type>,
        file_system_identifier: File_system_identifier_type,
    ) -> Result_type<&Internal_file_system_type> {
        file_systems
            .get(&file_system_identifier)
            .ok_or(Error_type::Invalid_identifier)
    }

    /// Mount a file system at a given mount point.
    pub async fn mount_file_system(
        &self,
        file_system: Box<dyn File_system_traits>,
        path: impl AsRef<Path_type>,
        task: Task_identifier_type,
    ) -> Result_type<File_system_identifier_type> {
        if !path.as_ref().is_valid() {
            return Err(Error_type::Invalid_path);
        }

        let path = path.as_ref();

        if !path.is_absolute() {
            return Err(Error_type::Invalid_path);
        }

        let mut file_systems = self.file_systems.write().await; // Get the file systems

        // Create a directory in the underlying file system
        let (_, parent_file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        parent_file_system.Create_directory(relative_path, time, user, group)?;

        // Create a directory at the mount point
        let file_system_identifier = Self::get_new_file_system_identifier(&file_systems)
            .ok_or(Error_type::Too_many_mounted_file_systems)?;

        file_systems.insert(
            file_system_identifier,
            Internal_file_system_type {
                mount_point: path.to_owned(),
                inner: file_system,
            },
        );

        Ok(file_system_identifier)
    }

    pub async fn unmount_file_system(
        &self,
        path: impl AsRef<Path_type>,
        task: Task_identifier_type,
    ) -> Result_type<()> {
        let path = path.as_ref();

        if !path.is_valid() || !path.is_absolute() {
            return Err(Error_type::Invalid_path);
        }

        let mut file_systems = self.file_systems.write().await; // Get the file systems

        let file_system_identifier = {
            let (file_system_identifier, _, relative_path) =
                Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

            if !relative_path.is_root() {
                return Err(Error_type::Invalid_path);
            }

            file_system_identifier
        };

        let file_system = file_systems
            .remove(&file_system_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        file_system.inner.Close_all(task)?;

        let (_, parent_file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &file_system.mount_point)?;

        parent_file_system.Remove(relative_path)?;

        Ok(())
    }

    fn get_file_system_from_path<'b>(
        file_systems: &'b BTreeMap<File_system_identifier_type, Internal_file_system_type>,
        path: &'b impl AsRef<Path_type>,
    ) -> Result_type<(
        File_system_identifier_type,
        &'b dyn File_system_traits,
        &'b Path_type,
    )> {
        let mut result_score = 0;
        let mut result: Option<File_system_identifier_type> = None;

        let path = path.as_ref();
        let path_components = path.get_components();

        for (file_system_identifier, file_system) in file_systems.iter() {
            let mount_point: &Path_type = file_system.mount_point.as_ref();
            let mount_point_components = mount_point.get_components();

            let score = path_components
                .clone()
                .get_common_components(mount_point_components);

            if result_score < score {
                result_score = score;
                result = Some(*file_system_identifier);
            }
        }

        let file_system_identifier = result.ok_or(Error_type::Invalid_path)?;

        let file_system = file_systems
            .get(&file_system_identifier)
            .ok_or(Error_type::Invalid_path)?;

        let relative_path = path
            .Strip_prefix_absolute(file_system.mount_point.as_ref())
            .ok_or(Error_type::Invalid_path)?;

        Ok((
            file_system_identifier,
            file_system.inner.as_ref(),
            relative_path,
        ))
    }

    pub async fn open(
        &self,
        path: &impl AsRef<Path_type>,
        flags: Flags_type,
        task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (file_system_identifier, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let time: Time_type = time::get_instance()
            .get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let local_file = file_system.Open(task, relative_path, flags, time, user, group)?;

        let metadata = file_system.get_metadata(local_file)?;

        let (_, unique_file) = local_file.Into_unique_file_identifier(file_system_identifier);

        let unique_file = match metadata.get_type() {
            Type_type::Character_device | Type_type::Block_device => {
                if let Some(inode) = metadata.get_inode() {
                    let local_file = self
                        .device_file_system
                        .Open(inode, task, flags, unique_file)
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
                if let Some(inode) = metadata.get_inode() {
                    let local_file = self
                        .pipe_file_system
                        .Open(inode, task, flags, unique_file)
                        .await?;

                    local_file
                        .Into_unique_file_identifier(File_system_identifier_type::PIPE_FILE_SYSTEM)
                        .1
                } else {
                    return Err(Error_type::Corrupted)?;
                }
            }
            _ => unique_file,
        };

        Ok(unique_file)
    }

    pub async fn close(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
    ) -> crate::Result_type<()> {
        let (file_system, local_file) = file.Into_local_file_identifier(task);

        let underlying_file = match file_system {
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
        let (file_system, local_file) = underlying_file.Into_local_file_identifier(task);

        self.file_systems
            .read()
            .await
            .get(&file_system)
            .ok_or(Error_type::Invalid_identifier)?
            .inner
            .Close(local_file)?;

        Ok(())
    }

    pub async fn read(
        &self,
        file: Unique_file_identifier_type,
        buffer: &mut [u8],
        task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (file_system, local_file_identifier) = file.Into_local_file_identifier(task);

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let (size, underlying_file) = match file_system {
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
                    .Read(local_file_identifier, buffer, time)
            }
        };

        if let Some(underlying_file) = underlying_file {
            let (file_system, local_file_identifier) =
                underlying_file.Into_local_file_identifier(task);

            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Read(local_file_identifier, &mut [0; 0], time)?;
        }
        Ok(size)
    }

    pub async fn read_line(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
        buffer: &mut String,
    ) -> Result_type<Size_type> {
        let (file_system, local_file_identifier) = file.Into_local_file_identifier(task);

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let (size, underlying_file) = match file_system {
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
                let file_systems = self.file_systems.read().await; // Get the file systems

                let file_system = &file_systems
                    .get(&file_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .inner;

                return read_line(&**file_system, buffer, local_file_identifier, time).await;
            }
        };

        if let Some(underlying_file) = underlying_file {
            let (file_system, local_file_identifier) =
                underlying_file.Into_local_file_identifier(task);

            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Read(local_file_identifier, &mut [0; 0], time)?;
        }

        Ok(size)
    }

    pub async fn read_to_end(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
        buffer: &mut Vec<u8>,
    ) -> Result_type<Size_type> {
        const CHUNK_SIZE: usize = 512;

        let mut read_size = 0;

        loop {
            let mut chunk = vec![0; CHUNK_SIZE];

            let size: usize = self.read(file, &mut chunk, task).await?.into();

            if size == 0 {
                break;
            }

            buffer.extend_from_slice(&chunk[..size]);

            read_size += size;
        }

        Ok(read_size.into())
    }

    pub async fn write(
        &self,
        file: Unique_file_identifier_type,
        buffer: &[u8],
        task: Task_identifier_type,
    ) -> Result_type<Size_type> {
        let (file_system, local_file_identifier) = file.Into_local_file_identifier(task);

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let (size, underlying_file) = match file_system {
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
                    .Write(local_file_identifier, buffer, time)
            }
        };

        if let Some(underlying_file) = underlying_file {
            let (file_system, local_file_identifier) =
                underlying_file.Into_local_file_identifier(task);

            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Write(local_file_identifier, &[0; 0], time)?;
        }

        Ok(size)
    }

    pub async fn set_position(
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

                let (file_system, local_file) = underlying_file.Into_local_file_identifier(task);

                self.file_systems
                    .read()
                    .await
                    .get(&file_system)
                    .ok_or(Error_type::Invalid_identifier)?
                    .inner
                    .Set_position(local_file, position)?;

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

    pub async fn set_owner(
        &self,
        path: impl AsRef<Path_type>,
        user: Option<User_identifier_type>,
        group: Option<Group_identifier_type>,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let mut metadata = file_system.get_metadata_from_path(relative_path)?;

        if let Some(user) = user {
            metadata.Set_owner(user);
        }

        if let Some(group) = group {
            metadata.set_group(group);
        }

        file_system.Set_metadata_from_path(relative_path, &metadata)
    }

    pub async fn set_permissions(
        &self,
        path: impl AsRef<Path_type>,
        permissions: Permissions_type,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let mut metadata = file_system.get_metadata_from_path(relative_path)?;

        metadata.Set_permissions(permissions);

        file_system.Set_metadata_from_path(relative_path, &metadata)
    }

    pub async fn close_all(&self, task_identifier: Task_identifier_type) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        for file_system in file_systems.values() {
            file_system.inner.Close_all(task_identifier)?;
        }

        self.device_file_system.Close_all(task_identifier).await?;

        self.pipe_file_system.Close_all(task_identifier).await?;

        Ok(())
    }

    pub async fn mount_device(
        &self,
        task: Task_identifier_type,
        path: &impl AsRef<Path_type>,
        device: Device_type,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let file = file_system.Open(
            task,
            relative_path,
            Flags_type::New(Mode_type::READ_WRITE, Some(Open_type::CREATE_ONLY), None),
            time,
            user,
            group,
        )?;

        file_system.Close(file)?;

        let inode = self
            .device_file_system
            .Mount_device(relative_path.to_owned(), device)
            .await?;

        let time: Time_type = time::get_instance()
            .get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let mut metadata = Metadata_type::get_default(Type_type::Block_device, time, user, group)
            .ok_or(Error_type::Invalid_parameter)?;
        metadata.Set_inode(inode);

        file_system.Set_metadata_from_path(relative_path, &metadata)?;

        Ok(())
    }

    pub async fn mount_static_device(
        &self,
        task: Task_identifier_type,
        path: &'a impl AsRef<Path_type>,
        device: Device_type,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        // Create a special file in the underlying file system.
        let (_, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let file = file_system.Open(
            task,
            relative_path,
            Flags_type::New(Mode_type::WRITE_ONLY, Some(Open_type::CREATE_ONLY), None),
            time,
            user,
            group,
        )?;

        file_system.Close(file)?;

        let r#type = if device.is_a_block_device() {
            Type_type::Block_device
        } else {
            Type_type::Character_device
        };

        // Create the actual device.
        let inode = self
            .device_file_system
            .Mount_static_device(path, device)
            .await?;

        let time: Time_type = time::get_instance()
            .get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        // Set the metadata of the special file.
        let mut metadata = Metadata_type::get_default(r#type, time, user, group)
            .ok_or(Error_type::Invalid_parameter)?;
        metadata.Set_inode(inode);

        file_system.Set_metadata_from_path(relative_path, &metadata)?;

        Ok(())
    }

    pub async fn create_named_pipe(
        &self,
        path: &impl AsRef<Path_type>,
        size: usize,
        task: Task_identifier_type,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, file_system, relative_path) = Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let file = file_system.Open(
            task,
            relative_path,
            Flags_type::New(Mode_type::READ_WRITE, Some(Open_type::CREATE_ONLY), None),
            time,
            user,
            group,
        )?;

        file_system.Close(file)?;

        let inode = self.pipe_file_system.Create_named_pipe(size).await?;

        let time: Time_type = time::get_instance()
            .get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let mut metadata = Metadata_type::get_default(Type_type::Pipe, time, user, group)
            .ok_or(Error_type::Invalid_parameter)?;
        metadata.Set_inode(inode);

        file_system.Set_metadata_from_path(relative_path, &metadata)?;

        Ok(())
    }

    pub async fn create_unnamed_pipe(
        &self,
        task: Task_identifier_type,
        status: Status_type,
        size: usize,
    ) -> Result_type<(Unique_file_identifier_type, Unique_file_identifier_type)> {
        let (read, write) = self
            .pipe_file_system
            .Create_unnamed_pipe(task, status, size)
            .await?;

        let (_, read) =
            read.Into_unique_file_identifier(File_system_identifier_type::PIPE_FILE_SYSTEM);
        let (_, write) =
            write.Into_unique_file_identifier(File_system_identifier_type::PIPE_FILE_SYSTEM);

        Ok((read, write))
    }

    pub async fn remove(&self, path: impl AsRef<Path_type>) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        // - Check metadata on the underlying file system
        let (_, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let metadata = file_system.get_metadata_from_path(relative_path)?;

        file_system.Remove(relative_path)?;

        match metadata.get_type() {
            Type_type::Pipe => {
                if let Some(inode) = metadata.get_inode() {
                    match self.pipe_file_system.Remove(inode).await {
                        Ok(_) | Err(Error_type::Invalid_inode) => (),
                        Err(error) => {
                            return Err(error);
                        }
                    }
                }
            }
            Type_type::Block_device => {
                if let Some(inode) = metadata.get_inode() {
                    match self.device_file_system.Remove(inode).await {
                        Ok(_) | Err(Error_type::Invalid_inode) => (),
                        Err(error) => {
                            return Err(error);
                        }
                    }
                }
            }
            Type_type::Character_device => {
                if let Some(inode) = metadata.get_inode() {
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

    pub async fn transfert(
        &self,
        file: Unique_file_identifier_type,
        current_task: Task_identifier_type,
        new_task: Task_identifier_type,
        new_file: Option<File_identifier_type>,
    ) -> Result_type<Unique_file_identifier_type> {
        let (file_system, file) = file.Into_local_file_identifier(current_task);

        let underlying_file = match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system.get_underlying_file(file).await?
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                Some(self.device_file_system.get_underlying_file(file).await?)
            }
            _ => None,
        };

        let file_systems = self.file_systems.read().await;

        let underlying_file = if let Some(underlying_file) = underlying_file {
            let (file_system, local_file) =
                underlying_file.Into_local_file_identifier(current_task);

            Some(
                file_systems
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

        let new_file = match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system
                    .Transfert(new_task, file, new_file)
                    .await?
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                let underlying_file = underlying_file.ok_or(Error_type::Internal_error)?;

                self.device_file_system
                    .Transfert(new_task, file, underlying_file, new_file)
                    .await?
            }
            _ => Self::get_file_system_from_identifier(&file_systems, file_system)?
                .inner
                .Transfert(new_task, file, new_file)?,
        };

        let (_, new_file) = new_file.Into_unique_file_identifier(file_system);

        Ok(new_file)
    }

    pub async fn flush(
        &self,
        file: Unique_file_identifier_type,
        task_identifier: Task_identifier_type,
    ) -> Result_type<()> {
        let (file_system, file_identifier) = file.Into_local_file_identifier(task_identifier);

        if file_system == File_system_identifier_type::PIPE_FILE_SYSTEM {
            Ok(())
        } else if file_system == File_system_identifier_type::DEVICE_FILE_SYSTEM {
            let underlying_file = self.device_file_system.Flush(file_identifier).await?;

            let (file_system, local_file) =
                underlying_file.Into_local_file_identifier(task_identifier);

            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .Flush(local_file)?;

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

    pub async fn get_statistics(
        &self,
        file: Unique_file_identifier_type,
        task_identifier: Task_identifier_type,
    ) -> Result_type<Statistics_type> {
        let (file_system, local_file) = file.Into_local_file_identifier(task_identifier);

        let file_systems = self.file_systems.read().await;

        let file = match file_system {
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                self.device_file_system
                    .get_underlying_file(local_file)
                    .await?
            }
            File_system_identifier_type::PIPE_FILE_SYSTEM => self
                .pipe_file_system
                .get_underlying_file(local_file)
                .await?
                .ok_or(Error_type::Unsupported_operation)?,
            _ => file,
        };

        let (file_system, local_file) = file.Into_local_file_identifier(task_identifier);

        Self::get_file_system_from_identifier(&file_systems, file_system)?
            .inner
            .get_statistics(local_file)
    }

    pub async fn open_directory(
        &self,
        path: &impl AsRef<Path_type>,
        task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (file_system_identifier, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let (_, file) = file_system
            .Open_directory(relative_path, task)?
            .Into_unique_file_identifier(file_system_identifier);

        Ok(file)
    }

    pub async fn read_directory(
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

    pub async fn set_position_directory(
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

    pub async fn get_position_directory(
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
                .get_position_directory(file),
        }
    }

    pub async fn rewind_directory(
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

    pub async fn close_directory(
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

    pub async fn create_directory(
        &self,
        path: &impl AsRef<Path_type>,
        task: Task_identifier_type,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, file_system, relative_path) = Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error_type::Time_error)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        file_system.Create_directory(relative_path, time, user, group)
    }

    pub async fn get_mode(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
    ) -> Result_type<Mode_type> {
        let (file_system, file) = file.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system.get_mode(file).await
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                self.device_file_system.get_mode(file).await
            }
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error_type::Invalid_identifier)?
                .inner
                .get_mode(file),
        }
    }

    pub async fn duplicate_file_identifier(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let (file_system, file) = file.Into_local_file_identifier(task);

        let underlying_file = match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system.get_underlying_file(file).await?
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                Some(self.device_file_system.get_underlying_file(file).await?)
            }
            _ => None,
        };

        let file_systems = self.file_systems.read().await;

        let underlying_file = if let Some(underlying_file) = underlying_file {
            let (file_system, local_file) = underlying_file.Into_local_file_identifier(task);

            Some(
                file_systems
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

        let new_file = match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => {
                self.pipe_file_system
                    .Duplicate(file, underlying_file)
                    .await?
            }
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                let underlying_file = underlying_file.ok_or(Error_type::Internal_error)?;

                self.device_file_system
                    .Duplicate(file, underlying_file)
                    .await?
            }
            _ => Self::get_file_system_from_identifier(&file_systems, file_system)?
                .inner
                .Duplicate(file)?,
        };

        let (_, new_file) = new_file.Into_unique_file_identifier(file_system);

        Ok(new_file)
    }

    pub async fn create_new_task_standard_io(
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
        let (standard_in, standard_error, standard_out) = if duplicate {
            let standard_in = self
                .duplicate_file_identifier(standard_in, current_task)
                .await?;
            let standard_error = self
                .duplicate_file_identifier(standard_error, current_task)
                .await?;
            let standard_out = self
                .duplicate_file_identifier(standard_out, current_task)
                .await?;

            (standard_in, standard_error, standard_out)
        } else {
            (standard_in, standard_error, standard_out)
        };

        let standard_in = self
            .transfert(
                standard_in,
                current_task,
                new_task,
                Some(File_identifier_type::STANDARD_IN),
            )
            .await?;
        let standard_error = self
            .transfert(
                standard_error,
                current_task,
                new_task,
                Some(File_identifier_type::STANDARD_ERROR),
            )
            .await?;
        let standard_out = self
            .transfert(
                standard_out,
                current_task,
                new_task,
                Some(File_identifier_type::STANDARD_OUT),
            )
            .await?;

        Ok((standard_in, standard_error, standard_out))
    }

    pub async fn is_a_terminal(
        &self,
        file: Unique_file_identifier_type,
        task: Task_identifier_type,
    ) -> Result_type<bool> {
        let (file_system, file) = file.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::PIPE_FILE_SYSTEM => Err(Error_type::Unsupported_operation),
            File_system_identifier_type::DEVICE_FILE_SYSTEM => {
                self.device_file_system.is_a_terminal(file).await
            }
            _ => Err(Error_type::Unsupported_operation),
        }
    }

    pub async fn rename(
        &self,
        old_path: &impl AsRef<Path_type>,
        new_path: &impl AsRef<Path_type>,
    ) -> Result_type<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (old_file_system_identifier, old_file_system, old_relative_path) =
            Self::get_file_system_from_path(&file_systems, old_path)?; // Get the file system identifier and the relative path

        let (new_file_system_identifier, _, new_relative_path) =
            Self::get_file_system_from_path(&file_systems, new_path)?; // Get the file system identifier and the relative path

        if old_file_system_identifier != new_file_system_identifier {
            return Err(Error_type::Invalid_path);
        }

        if old_file_system_identifier == new_file_system_identifier {
            old_file_system.Rename(old_relative_path, new_relative_path)
        } else {
            Err(Error_type::Unsupported_operation) // TODO : Add support for moving between file systems
        }
    }

    pub async fn get_raw_device(&self, path: &impl AsRef<Path_type>) -> Result_type<Device_type> {
        let file_systems = self.file_systems.read().await;

        let (_, file_system, relative_path) = Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let metadata = file_system.get_metadata_from_path(relative_path)?;

        if metadata.get_type() != Type_type::Block_device
            && metadata.get_type() != Type_type::Character_device
        {
            return Err(Error_type::Unsupported_operation);
        }

        if let Some(inode) = metadata.get_inode() {
            self.device_file_system.get_raw_device(inode).await
        } else {
            Err(Error_type::Corrupted)
        }
    }

    pub async fn get_metadata_from_path(
        &self,
        path: &impl AsRef<Path_type>,
    ) -> Result_type<Metadata_type> {
        let file_systems = self.file_systems.read().await;

        let (_, file_system, relative_path) = Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        file_system.get_metadata_from_path(relative_path)
    }

    pub async fn get_statistics_from_path(
        &self,
        path: &impl AsRef<Path_type>,
    ) -> Result_type<Statistics_type> {
        let file_systems = self.file_systems.read().await;

        let (file_system_identifier, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let metadata = file_system.get_metadata_from_path(relative_path)?;

        Ok(Statistics_type::new(
            file_system_identifier,
            metadata.get_inode().unwrap_or(Inode_type::New(0)),
            0,
            Size_type::New(0),
            metadata.get_access_time(),
            metadata.get_modification_time(),
            metadata.get_creation_time(),
            metadata.get_type(),
            metadata.get_permissions(),
            metadata.get_user(),
            metadata.get_group(),
        ))
    }

    pub async fn send(
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

    pub async fn receive(
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

    pub async fn send_to(
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

    pub async fn receive_from(
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

    fn new_file_identifier(
        &self,
        file_system: File_system_identifier_type,
        task: Task_identifier_type,
    ) -> crate::Result_type<Local_file_identifier_type> {
        let iterator = Local_file_identifier_type::get_minimum(task).into_iter();

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => Ok(self
                .network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .get_new_socket_identifier(iterator)?
                .ok_or(crate::Error_type::Too_many_open_files)?),
            _ => Err(crate::Error_type::Invalid_file_system)?,
        }
    }

    pub async fn bind(
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

        let new_socket = self.new_file_identifier(file_system, task)?;

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => {
                let (ip, port) = if let Some((ip_type, port)) = address.into_ip_and_port() {
                    (ip_type, port)
                } else {
                    unreachable!()
                };

                self.network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Bind(ip, port, protocol, new_socket)?
            }
            _ => return Err(crate::Error_type::Invalid_file_system),
        }

        let (_, new_socket) = new_socket.Into_unique_file_identifier(file_system);

        Ok(new_socket)
    }

    pub async fn connect(
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

        let new_socket = self.new_file_identifier(file_system, task)?;

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => {
                let (ip, port) = if let Some((ip_type, port)) = address.into_ip_and_port() {
                    (ip_type, port)
                } else {
                    unreachable!()
                };

                self.network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Connect(ip, port, new_socket)?
            }
            _ => return Err(crate::Error_type::Invalid_file_system),
        }

        let (_, new_socket) = new_socket.Into_unique_file_identifier(file_system);

        Ok(new_socket)
    }

    pub async fn accept(
        &self,
        task: Task_identifier_type,
        socket: Unique_file_identifier_type,
    ) -> crate::Result_type<(Unique_file_identifier_type, Option<(IP_type, Port_type)>)> {
        let (file_system, socket) = socket.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => {
                let new_socket = self.new_file_identifier(file_system, task)?;

                let (ip, port) = self
                    .network_socket_driver
                    .ok_or(crate::Error_type::Unavailable_driver)?
                    .Accept(socket, new_socket)?;

                let (_, new_socket) = new_socket.Into_unique_file_identifier(file_system);

                Ok((new_socket, Some((ip, port))))
            }
            _ => Err(crate::Error_type::Invalid_file_system),
        }
    }

    pub async fn set_send_timeout(
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

    pub async fn set_receive_timeout(
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

    pub async fn get_send_timeout(
        &self,
        task: Task_identifier_type,
        socket: Unique_file_identifier_type,
    ) -> crate::Result_type<Option<Duration_type>> {
        let (file_system, socket) = socket.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => Ok(self
                .network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .get_send_timeout(socket)?),
            _ => Err(crate::Error_type::Invalid_file_system),
        }
    }

    pub async fn get_receive_timeout(
        &self,
        task: Task_identifier_type,
        socket: Unique_file_identifier_type,
    ) -> crate::Result_type<Option<Duration_type>> {
        let (file_system, socket) = socket.Into_local_file_identifier(task);

        match file_system {
            File_system_identifier_type::NETWORK_SOCKET_FILE_SYSTEM => Ok(self
                .network_socket_driver
                .ok_or(crate::Error_type::Unavailable_driver)?
                .get_receive_timeout(socket)?),
            _ => Err(crate::Error_type::Invalid_file_system),
        }
    }
}

async fn read_line(
    file_system: &dyn File_system_traits,
    buffer: &mut String,
    file: Local_file_identifier_type,
    time: Time_type,
) -> Result_type<Size_type> {
    loop {
        let current_buffer = &mut [0; 1];

        let size = file_system.Read(file, current_buffer, time)?;

        if size == 0 {
            yield_now().await; // Yield to allow other tasks to run, especially in a blocking operation
            continue; // Retry reading if no data was read
        }

        let byte = current_buffer[0];

        if byte == b'\n' || byte == b'\r' {
            break;
        }

        buffer.push(byte as char);
    }

    Ok(buffer.len().into())
}

#[cfg(test)]
mod tests {
    use file_system::Local_file_identifier_type;

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

        fn get_position_directory(&self, _: Local_file_identifier_type) -> Result_type<Size_type> {
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

        fn get_metadata_from_path(&self, _: &Path_type) -> Result_type<Metadata_type> {
            todo!()
        }

        fn get_statistics(&self, _: Local_file_identifier_type) -> Result_type<Statistics_type> {
            todo!()
        }

        fn get_mode(&self, _: Local_file_identifier_type) -> Result_type<Mode_type> {
            todo!()
        }

        fn get_metadata(&self, _: Local_file_identifier_type) -> Result_type<Metadata_type> {
            todo!()
        }
    }

    #[test]
    fn test_get_file_system_from_path() {
        let mut file_systems: BTreeMap<File_system_identifier_type, Internal_file_system_type> =
            BTreeMap::new();

        file_systems.insert(
            1.into(),
            Internal_file_system_type {
                mount_point: Path_owned_type::New("/".to_string()).unwrap(),
                inner: Box::new(Dummy_file_system_type),
            },
        );

        file_systems.insert(
            2.into(),
            Internal_file_system_type {
                mount_point: Path_owned_type::New("/Foo".to_string()).unwrap(),
                inner: Box::new(Dummy_file_system_type),
            },
        );

        file_systems.insert(
            3.into(),
            Internal_file_system_type {
                mount_point: Path_owned_type::New("/Foo/Bar".to_string()).unwrap(),
                inner: Box::new(Dummy_file_system_type),
            },
        );

        let (file_system, _, relative_path) =
            Virtual_file_system_type::get_file_system_from_path(&file_systems, &"/").unwrap();

        assert_eq!(file_system, 1.into());
        assert_eq!(relative_path, Path_type::ROOT);

        let (file_system, _, relative_path) =
            Virtual_file_system_type::get_file_system_from_path(&file_systems, &"/Foo/Bar")
                .unwrap();

        assert_eq!(file_system, 3.into());
        assert_eq!(relative_path, Path_type::ROOT);

        let (file_system, _, relative_path) =
            Virtual_file_system_type::get_file_system_from_path(&file_systems, &"/Foo/Bar/Baz")
                .unwrap();

        assert_eq!(file_system, 3.into());
        assert_eq!(relative_path, "/Baz".as_ref());

        let (file_system, _, relative_path) =
            Virtual_file_system_type::get_file_system_from_path(&file_systems, &"/Foo").unwrap();

        assert_eq!(file_system, 2.into());
        assert_eq!(relative_path, Path_type::ROOT);
    }
}
