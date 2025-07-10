use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use futures::yield_now;
use synchronization::{
    blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, rwlock::RwLock,
};

use alloc::{boxed::Box, collections::BTreeMap};

use network::{Port, Protocol, SocketDriver, IP};
use task::TaskIdentifier;
use time::Duration;
use users::{GroupIdentifier, UserIdentifier};

use file_system::{
    DeviceType, Entry, FileIdentifier, Inode, Kind, LocalFileIdentifier, Metadata, Mode, Open,
    Statistics_type, Time,
};

use file_system::{
    Error, FileSystemIdentifier, FileSystemTraits, Flags, Path, PathOwned, Permissions, Position,
    Result, Size, Status, UniqueFileIdentifier,
};

use crate::device::InternalPathType;
use crate::{device, pipe, SockerAddress};

struct InternalFileSystemType {
    pub mount_point: PathOwned,
    pub inner: Box<dyn FileSystemTraits>,
}

/// Instance of the virtual file system.
///
/// # Safety
/// I know, it is not safe to use mutable static variables.
/// It is thread safe (after initialization) because it is only read after initialization.
/// It is a pragmatic choice for efficiency in embedded systems contexts (avoid using Arc).
static VIRTUAL_FILE_SYSTEM_INSTANCE: OnceLock<VirtualFileSystemType> = OnceLock::new();

pub fn initialize(
    root_file_system: Box<dyn FileSystemTraits>,
    network_socket_driver: Option<&'static dyn SocketDriver>,
) -> Result<&'static VirtualFileSystemType<'static>> {
    let virtual_file_system = VirtualFileSystemType::new(
        task::get_instance(),
        users::get_instance(),
        time::get_instance(),
        root_file_system,
        network_socket_driver,
    )?;

    Ok(VIRTUAL_FILE_SYSTEM_INSTANCE.get_or_init(|| virtual_file_system))
}

pub fn get_instance() -> &'static VirtualFileSystemType<'static> {
    VIRTUAL_FILE_SYSTEM_INSTANCE
        .try_get()
        .expect("Virtual file system not initialized")
}

/// The virtual file system.
///
/// It is a singleton.
pub struct VirtualFileSystemType<'a> {
    /// Mounted file systems.
    file_systems:
        RwLock<CriticalSectionRawMutex, BTreeMap<FileSystemIdentifier, InternalFileSystemType>>,
    /// Devices.
    device_file_system: device::FileSystemType<'a>,
    /// Pipes.
    pipe_file_system: pipe::FileSystemType,
    /// Network sockets.
    network_socket_driver: Option<&'a dyn SocketDriver>,
}

impl<'a> VirtualFileSystemType<'a> {
    pub const STANDARD_INPUT_FILE_IDENTIFIER: FileIdentifier = FileIdentifier::new(0);
    pub const STANDARD_OUTPUT_FILE_IDENTIFIER: FileIdentifier = FileIdentifier::new(1);
    pub const STANDARD_ERROR_FILE_IDENTIFIER: FileIdentifier = FileIdentifier::new(2);

    pub fn new(
        _: &'static task::Manager,
        _: &'static users::Manager,
        _: &'static time::Manager,
        root_file_system: Box<dyn FileSystemTraits>,
        network_socket_driver: Option<&'a dyn SocketDriver>,
    ) -> Result<Self> {
        let mut file_systems = BTreeMap::new();

        let identifier = Self::get_new_file_system_identifier(&file_systems)
            .ok_or(Error::TooManyMountedFileSystems)?;

        file_systems.insert(
            identifier,
            InternalFileSystemType {
                mount_point: PathOwned::new("/".to_string()).unwrap(),
                inner: root_file_system,
            },
        );

        Ok(Self {
            file_systems: RwLock::new(file_systems),
            device_file_system: device::FileSystemType::new(),
            pipe_file_system: pipe::FileSystemType::new(),
            network_socket_driver,
        })
    }

    pub async fn uninitialize(&self) {
        if let Ok(inodes) = self
            .device_file_system
            .get_devices_from_path(Path::ROOT)
            .await
        {
            for inode in inodes {
                if let Ok(path) = self.device_file_system.get_path_from_inode(inode).await {
                    match path {
                        InternalPathType::Owned(path) => {
                            let _ = self.remove(path).await;
                        }
                        InternalPathType::Borrowed(path) => {
                            let _ = self.remove(path).await;
                        }
                    }
                }
            }
        }
    }

    fn get_new_file_system_identifier(
        file_systems: &BTreeMap<FileSystemIdentifier, InternalFileSystemType>,
    ) -> Option<FileSystemIdentifier> {
        let mut file_system_identifier = FileSystemIdentifier::MINIMUM;

        while file_systems.contains_key(&file_system_identifier) {
            file_system_identifier += 1;
        }

        Some(file_system_identifier)
    }

    fn get_file_system_from_identifier(
        file_systems: &BTreeMap<FileSystemIdentifier, InternalFileSystemType>,
        file_system_identifier: FileSystemIdentifier,
    ) -> Result<&InternalFileSystemType> {
        file_systems
            .get(&file_system_identifier)
            .ok_or(Error::InvalidIdentifier)
    }

    /// Mount a file system at a given mount point.
    pub async fn mount_file_system(
        &self,
        file_system: Box<dyn FileSystemTraits>,
        path: impl AsRef<Path>,
        task: TaskIdentifier,
    ) -> Result<FileSystemIdentifier> {
        if !path.as_ref().is_valid() {
            return Err(Error::InvalidPath);
        }

        let path = path.as_ref();

        if !path.is_absolute() {
            return Err(Error::InvalidPath);
        }

        let mut file_systems = self.file_systems.write().await; // Get the file systems

        // Create a directory in the underlying file system
        let (_, parent_file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error::TimeError)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        parent_file_system.create_directory(relative_path, time, user, group)?;

        // Create a directory at the mount point
        let file_system_identifier = Self::get_new_file_system_identifier(&file_systems)
            .ok_or(Error::TooManyMountedFileSystems)?;

        file_systems.insert(
            file_system_identifier,
            InternalFileSystemType {
                mount_point: path.to_owned(),
                inner: file_system,
            },
        );

        Ok(file_system_identifier)
    }

    pub async fn unmount_file_system(
        &self,
        path: impl AsRef<Path>,
        task: TaskIdentifier,
    ) -> Result<()> {
        let path = path.as_ref();

        if !path.is_valid() || !path.is_absolute() {
            return Err(Error::InvalidPath);
        }

        let mut file_systems = self.file_systems.write().await; // Get the file systems

        let file_system_identifier = {
            let (file_system_identifier, _, relative_path) =
                Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

            if !relative_path.is_root() {
                return Err(Error::InvalidPath);
            }

            file_system_identifier
        };

        let file_system = file_systems
            .remove(&file_system_identifier)
            .ok_or(Error::InvalidIdentifier)?;

        file_system.inner.close_all(task)?;

        let (_, parent_file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &file_system.mount_point)?;

        parent_file_system.remove(relative_path)?;

        Ok(())
    }

    fn get_file_system_from_path<'b>(
        file_systems: &'b BTreeMap<FileSystemIdentifier, InternalFileSystemType>,
        path: &'b impl AsRef<Path>,
    ) -> Result<(FileSystemIdentifier, &'b dyn FileSystemTraits, &'b Path)> {
        let mut result_score = 0;
        let mut result: Option<FileSystemIdentifier> = None;

        let path = path.as_ref();
        let path_components = path.get_components();

        for (file_system_identifier, file_system) in file_systems.iter() {
            let mount_point: &Path = file_system.mount_point.as_ref();
            let mount_point_components = mount_point.get_components();

            let score = path_components
                .clone()
                .get_common_components(mount_point_components);

            if result_score < score {
                result_score = score;
                result = Some(*file_system_identifier);
            }
        }

        let file_system_identifier = result.ok_or(Error::InvalidPath)?;

        let file_system = file_systems
            .get(&file_system_identifier)
            .ok_or(Error::InvalidPath)?;

        let relative_path = path
            .strip_prefix_absolute(file_system.mount_point.as_ref())
            .ok_or(Error::InvalidPath)?;

        Ok((
            file_system_identifier,
            file_system.inner.as_ref(),
            relative_path,
        ))
    }

    pub async fn open(
        &self,
        path: &impl AsRef<Path>,
        flags: Flags,
        task: TaskIdentifier,
    ) -> Result<UniqueFileIdentifier> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (file_system_identifier, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let time: Time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error::TimeError)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let local_file = file_system.open(task, relative_path, flags, time, user, group)?;

        let metadata = file_system.get_metadata(local_file)?;

        let (_, unique_file) = local_file.into_unique_file_identifier(file_system_identifier);

        let unique_file = match metadata.get_type() {
            Kind::CharacterDevice | Kind::BlockDevice => {
                if let Some(inode) = metadata.get_inode() {
                    let local_file = self
                        .device_file_system
                        .open(inode, task, flags, unique_file)
                        .await?;

                    local_file
                        .into_unique_file_identifier(FileSystemIdentifier::DEVICE_FILE_SYSTEM)
                        .1
                } else {
                    return Err(Error::Corrupted)?;
                }
            }
            Kind::Pipe => {
                if let Some(inode) = metadata.get_inode() {
                    let local_file = self
                        .pipe_file_system
                        .open(inode, task, flags, unique_file)
                        .await?;

                    local_file
                        .into_unique_file_identifier(FileSystemIdentifier::PIPE_FILE_SYSTEM)
                        .1
                } else {
                    return Err(Error::Corrupted)?;
                }
            }
            _ => unique_file,
        };

        Ok(unique_file)
    }

    pub async fn close(
        &self,
        file: UniqueFileIdentifier,
        task: TaskIdentifier,
    ) -> crate::Result<()> {
        let (file_system, local_file) = file.into_local_file_identifier(task);

        let underlying_file = match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => {
                match self.pipe_file_system.close(local_file).await? {
                    Some(underlying_file) => underlying_file,
                    None => return Ok(()),
                }
            }
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => {
                self.device_file_system.close(local_file).await?
            }
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => {
                self.network_socket_driver
                    .ok_or(Error::UnsupportedOperation)?
                    .close(local_file)?;

                return Ok(());
            }
            _ => {
                self.file_systems
                    .read()
                    .await
                    .get(&file_system)
                    .ok_or(Error::InvalidIdentifier)?
                    .inner
                    .close(local_file)?;

                return Ok(());
            }
        };

        // - If there is an underlying file (some pipe and devices), close it too.
        let (file_system, local_file) = underlying_file.into_local_file_identifier(task);

        self.file_systems
            .read()
            .await
            .get(&file_system)
            .ok_or(Error::InvalidIdentifier)?
            .inner
            .close(local_file)?;

        Ok(())
    }

    pub async fn read(
        &self,
        file: UniqueFileIdentifier,
        buffer: &mut [u8],
        task: TaskIdentifier,
    ) -> Result<Size> {
        let (file_system, local_file_identifier) = file.into_local_file_identifier(task);

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error::TimeError)?
            .into();

        let (size, underlying_file) = match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => {
                self.pipe_file_system
                    .read(local_file_identifier, buffer)
                    .await?
            }
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => {
                let result = self
                    .device_file_system
                    .read(local_file_identifier, buffer)
                    .await?;
                (result.0, Some(result.1))
            }
            _ => {
                return self
                    .file_systems
                    .read()
                    .await
                    .get(&file_system)
                    .ok_or(Error::InvalidIdentifier)?
                    .inner
                    .read(local_file_identifier, buffer, time)
            }
        };

        if let Some(underlying_file) = underlying_file {
            let (file_system, local_file_identifier) =
                underlying_file.into_local_file_identifier(task);

            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error::InvalidIdentifier)?
                .inner
                .read(local_file_identifier, &mut [0; 0], time)?;
        }
        Ok(size)
    }

    pub async fn read_line(
        &self,
        file: UniqueFileIdentifier,
        task: TaskIdentifier,
        buffer: &mut String,
    ) -> Result<Size> {
        let (file_system, local_file_identifier) = file.into_local_file_identifier(task);

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error::TimeError)?
            .into();

        let (size, underlying_file) = match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => {
                self.pipe_file_system
                    .read_line(local_file_identifier, buffer)
                    .await?
            }
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => {
                let result = self
                    .device_file_system
                    .read_line(local_file_identifier, buffer)
                    .await?;
                (result.0, Some(result.1))
            }
            _ => {
                let file_systems = self.file_systems.read().await; // Get the file systems

                let file_system = &file_systems
                    .get(&file_system)
                    .ok_or(Error::InvalidIdentifier)?
                    .inner;

                return read_line(&**file_system, buffer, local_file_identifier, time).await;
            }
        };

        if let Some(underlying_file) = underlying_file {
            let (file_system, local_file_identifier) =
                underlying_file.into_local_file_identifier(task);

            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error::InvalidIdentifier)?
                .inner
                .read(local_file_identifier, &mut [0; 0], time)?;
        }

        Ok(size)
    }

    pub async fn read_to_end(
        &self,
        file: UniqueFileIdentifier,
        task: TaskIdentifier,
        buffer: &mut Vec<u8>,
    ) -> Result<Size> {
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
        file: UniqueFileIdentifier,
        buffer: &[u8],
        task: TaskIdentifier,
    ) -> Result<Size> {
        let (file_system, local_file_identifier) = file.into_local_file_identifier(task);

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error::TimeError)?
            .into();

        let (size, underlying_file) = match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => {
                self.pipe_file_system
                    .write(local_file_identifier, buffer)
                    .await?
            }
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => {
                let result = self
                    .device_file_system
                    .write(local_file_identifier, buffer)
                    .await?;
                (result.0, Some(result.1))
            }
            _ => {
                return self
                    .file_systems
                    .read()
                    .await
                    .get(&file_system)
                    .ok_or(Error::InvalidIdentifier)?
                    .inner
                    .write(local_file_identifier, buffer, time)
            }
        };

        if let Some(underlying_file) = underlying_file {
            let (file_system, local_file_identifier) =
                underlying_file.into_local_file_identifier(task);

            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error::InvalidIdentifier)?
                .inner
                .write(local_file_identifier, &[0; 0], time)?;
        }

        Ok(size)
    }

    pub async fn set_position(
        &self,
        file: UniqueFileIdentifier,
        position: &Position,
        task: TaskIdentifier,
    ) -> Result<Size> {
        let (file_system, local_file) = file.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => Err(Error::UnsupportedOperation),
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => {
                let (size, underlying_file) = self
                    .device_file_system
                    .set_position(local_file, position)
                    .await?;

                let (file_system, local_file) = underlying_file.into_local_file_identifier(task);

                self.file_systems
                    .read()
                    .await
                    .get(&file_system)
                    .ok_or(Error::InvalidIdentifier)?
                    .inner
                    .set_position(local_file, position)?;

                Ok(size)
            }
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error::InvalidIdentifier)?
                .inner
                .set_position(local_file, position),
        }
    }

    pub async fn set_owner(
        &self,
        path: impl AsRef<Path>,
        user: Option<UserIdentifier>,
        group: Option<GroupIdentifier>,
    ) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let mut metadata = file_system.get_metadata_from_path(relative_path)?;

        if let Some(user) = user {
            metadata.set_owner(user);
        }

        if let Some(group) = group {
            metadata.set_group(group);
        }

        file_system.set_metadata_from_path(relative_path, &metadata)
    }

    pub async fn set_permissions(
        &self,
        path: impl AsRef<Path>,
        permissions: Permissions,
    ) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let mut metadata = file_system.get_metadata_from_path(relative_path)?;

        metadata.set_permissions(permissions);

        file_system.set_metadata_from_path(relative_path, &metadata)
    }

    pub async fn close_all(&self, task_identifier: TaskIdentifier) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        for file_system in file_systems.values() {
            file_system.inner.close_all(task_identifier)?;
        }

        self.device_file_system.close_all(task_identifier).await?;

        self.pipe_file_system.close_all(task_identifier).await?;

        Ok(())
    }

    pub async fn mount_device(
        &self,
        task: TaskIdentifier,
        path: &impl AsRef<Path>,
        device: DeviceType,
    ) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error::TimeError)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let file = file_system.open(
            task,
            relative_path,
            Flags::new(Mode::READ_WRITE, Some(Open::CREATE_ONLY), None),
            time,
            user,
            group,
        )?;

        file_system.close(file)?;

        let inode = self
            .device_file_system
            .mount_device(relative_path.to_owned(), device)
            .await?;

        let time: Time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error::TimeError)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let mut metadata = Metadata::get_default(Kind::BlockDevice, time, user, group)
            .ok_or(Error::InvalidParameter)?;
        metadata.set_inode(inode);

        file_system.set_metadata_from_path(relative_path, &metadata)?;

        Ok(())
    }

    pub async fn mount_static_device(
        &self,
        task: TaskIdentifier,
        path: &'a impl AsRef<Path>,
        device: DeviceType,
    ) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        // Create a special file in the underlying file system.
        let (_, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error::TimeError)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let file = file_system.open(
            task,
            relative_path,
            Flags::new(Mode::WRITE_ONLY, Some(Open::CREATE_ONLY), None),
            time,
            user,
            group,
        )?;

        file_system.close(file)?;

        let r#type = if device.is_a_block_device() {
            Kind::BlockDevice
        } else {
            Kind::CharacterDevice
        };

        // Create the actual device.
        let inode = self
            .device_file_system
            .mount_static_device(path, device)
            .await?;

        let time: Time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error::TimeError)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        // Set the metadata of the special file.
        let mut metadata =
            Metadata::get_default(r#type, time, user, group).ok_or(Error::InvalidParameter)?;
        metadata.set_inode(inode);

        file_system.set_metadata_from_path(relative_path, &metadata)?;

        Ok(())
    }

    pub async fn create_named_pipe(
        &self,
        path: &impl AsRef<Path>,
        size: usize,
        task: TaskIdentifier,
    ) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, file_system, relative_path) = Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error::TimeError)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let file = file_system.open(
            task,
            relative_path,
            Flags::new(Mode::READ_WRITE, Some(Open::CREATE_ONLY), None),
            time,
            user,
            group,
        )?;

        file_system.close(file)?;

        let inode = self.pipe_file_system.create_named_pipe(size).await?;

        let time: Time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error::TimeError)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let mut metadata =
            Metadata::get_default(Kind::Pipe, time, user, group).ok_or(Error::InvalidParameter)?;
        metadata.set_inode(inode);

        file_system.set_metadata_from_path(relative_path, &metadata)?;

        Ok(())
    }

    pub async fn create_unnamed_pipe(
        &self,
        task: TaskIdentifier,
        status: Status,
        size: usize,
    ) -> Result<(UniqueFileIdentifier, UniqueFileIdentifier)> {
        let (read, write) = self
            .pipe_file_system
            .create_unnamed_pipe(task, status, size)
            .await?;

        let (_, read) = read.into_unique_file_identifier(FileSystemIdentifier::PIPE_FILE_SYSTEM);
        let (_, write) = write.into_unique_file_identifier(FileSystemIdentifier::PIPE_FILE_SYSTEM);

        Ok((read, write))
    }

    pub async fn remove(&self, path: impl AsRef<Path>) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        // - Check metadata on the underlying file system
        let (_, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let metadata = file_system.get_metadata_from_path(relative_path)?;

        file_system.remove(relative_path)?;

        match metadata.get_type() {
            Kind::Pipe => {
                if let Some(inode) = metadata.get_inode() {
                    match self.pipe_file_system.remove(inode).await {
                        Ok(_) | Err(Error::InvalidInode) => (),
                        Err(error) => {
                            return Err(error);
                        }
                    }
                }
            }
            Kind::BlockDevice => {
                if let Some(inode) = metadata.get_inode() {
                    match self.device_file_system.remove(inode).await {
                        Ok(_) | Err(Error::InvalidInode) => (),
                        Err(error) => {
                            return Err(error);
                        }
                    }
                }
            }
            Kind::CharacterDevice => {
                if let Some(inode) = metadata.get_inode() {
                    match self.device_file_system.remove(inode).await {
                        Ok(_) | Err(Error::InvalidInode) => (),
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
        file: UniqueFileIdentifier,
        current_task: TaskIdentifier,
        new_task: TaskIdentifier,
        new_file: Option<FileIdentifier>,
    ) -> Result<UniqueFileIdentifier> {
        let (file_system, file) = file.into_local_file_identifier(current_task);

        let underlying_file = match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => {
                self.pipe_file_system.get_underlying_file(file).await?
            }
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => {
                Some(self.device_file_system.get_underlying_file(file).await?)
            }
            _ => None,
        };

        let file_systems = self.file_systems.read().await;

        let underlying_file = if let Some(underlying_file) = underlying_file {
            let (file_system, local_file) =
                underlying_file.into_local_file_identifier(current_task);

            Some(
                file_systems
                    .get(&file_system)
                    .ok_or(Error::InvalidIdentifier)?
                    .inner
                    .transfert(new_task, local_file, new_file)?
                    .into_unique_file_identifier(file_system)
                    .1,
            )
        } else {
            None
        };

        let new_file = match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => {
                self.pipe_file_system
                    .transfert(new_task, file, new_file)
                    .await?
            }
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => {
                let underlying_file = underlying_file.ok_or(Error::InternalError)?;

                self.device_file_system
                    .transfert(new_task, file, underlying_file, new_file)
                    .await?
            }
            _ => Self::get_file_system_from_identifier(&file_systems, file_system)?
                .inner
                .transfert(new_task, file, new_file)?,
        };

        let (_, new_file) = new_file.into_unique_file_identifier(file_system);

        Ok(new_file)
    }

    pub async fn flush(
        &self,
        file: UniqueFileIdentifier,
        task_identifier: TaskIdentifier,
    ) -> Result<()> {
        let (file_system, file_identifier) = file.into_local_file_identifier(task_identifier);

        if file_system == FileSystemIdentifier::PIPE_FILE_SYSTEM {
            Ok(())
        } else if file_system == FileSystemIdentifier::DEVICE_FILE_SYSTEM {
            let underlying_file = self.device_file_system.flush(file_identifier).await?;

            let (file_system, local_file) =
                underlying_file.into_local_file_identifier(task_identifier);

            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error::InvalidIdentifier)?
                .inner
                .flush(local_file)?;

            Ok(())
        } else {
            self.file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error::InvalidIdentifier)?
                .inner
                .flush(file_identifier)
        }
    }

    pub async fn get_statistics(
        &self,
        file: UniqueFileIdentifier,
        task_identifier: TaskIdentifier,
    ) -> Result<Statistics_type> {
        let (file_system, local_file) = file.into_local_file_identifier(task_identifier);

        let file_systems = self.file_systems.read().await;

        let file = match file_system {
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => {
                self.device_file_system
                    .get_underlying_file(local_file)
                    .await?
            }
            FileSystemIdentifier::PIPE_FILE_SYSTEM => self
                .pipe_file_system
                .get_underlying_file(local_file)
                .await?
                .ok_or(Error::UnsupportedOperation)?,
            _ => file,
        };

        let (file_system, local_file) = file.into_local_file_identifier(task_identifier);

        Self::get_file_system_from_identifier(&file_systems, file_system)?
            .inner
            .get_statistics(local_file)
    }

    pub async fn open_directory(
        &self,
        path: &impl AsRef<Path>,
        task: TaskIdentifier,
    ) -> Result<UniqueFileIdentifier> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (file_system_identifier, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let (_, file) = file_system
            .open_directory(relative_path, task)?
            .into_unique_file_identifier(file_system_identifier);

        Ok(file)
    }

    pub async fn read_directory(
        &self,
        file: UniqueFileIdentifier,
        task: TaskIdentifier,
    ) -> Result<Option<Entry>> {
        let (file_system, file) = file.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => Err(Error::UnsupportedOperation),
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => Err(Error::UnsupportedOperation),
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error::InvalidIdentifier)?
                .inner
                .read_directory(file),
        }
    }

    pub async fn set_position_directory(
        &self,
        file: UniqueFileIdentifier,
        position: Size,
        task: TaskIdentifier,
    ) -> Result<()> {
        let (file_system, file) = file.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => Err(Error::UnsupportedOperation),
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => Err(Error::UnsupportedOperation),
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error::InvalidIdentifier)?
                .inner
                .set_position_directory(file, position),
        }
    }

    pub async fn get_position_directory(
        &self,
        file: UniqueFileIdentifier,
        task: TaskIdentifier,
    ) -> Result<Size> {
        let (file_system, file) = file.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => Err(Error::UnsupportedOperation),
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => Err(Error::UnsupportedOperation),
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error::InvalidIdentifier)?
                .inner
                .get_position_directory(file),
        }
    }

    pub async fn rewind_directory(
        &self,
        file: UniqueFileIdentifier,
        task: TaskIdentifier,
    ) -> Result<()> {
        let (file_system, file) = file.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => Err(Error::UnsupportedOperation),
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => Err(Error::UnsupportedOperation),
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error::InvalidIdentifier)?
                .inner
                .rewind_directory(file),
        }
    }

    pub async fn close_directory(
        &self,
        file: UniqueFileIdentifier,
        task: TaskIdentifier,
    ) -> Result<()> {
        let (file_system, file) = file.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => Err(Error::UnsupportedOperation),
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => Err(Error::UnsupportedOperation),
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error::InvalidIdentifier)?
                .inner
                .close_directory(file),
        }
    }

    pub async fn create_directory(
        &self,
        path: &impl AsRef<Path>,
        task: TaskIdentifier,
    ) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (_, file_system, relative_path) = Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let time = time::get_instance()
            .get_current_time()
            .map_err(|_| Error::TimeError)?
            .into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        file_system.create_directory(relative_path, time, user, group)
    }

    pub async fn get_mode(&self, file: UniqueFileIdentifier, task: TaskIdentifier) -> Result<Mode> {
        let (file_system, file) = file.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => self.pipe_file_system.get_mode(file).await,
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => {
                self.device_file_system.get_mode(file).await
            }
            _ => self
                .file_systems
                .read()
                .await
                .get(&file_system)
                .ok_or(Error::InvalidIdentifier)?
                .inner
                .get_mode(file),
        }
    }

    pub async fn duplicate_file_identifier(
        &self,
        file: UniqueFileIdentifier,
        task: TaskIdentifier,
    ) -> Result<UniqueFileIdentifier> {
        let (file_system, file) = file.into_local_file_identifier(task);

        let underlying_file = match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => {
                self.pipe_file_system.get_underlying_file(file).await?
            }
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => {
                Some(self.device_file_system.get_underlying_file(file).await?)
            }
            _ => None,
        };

        let file_systems = self.file_systems.read().await;

        let underlying_file = if let Some(underlying_file) = underlying_file {
            let (file_system, local_file) = underlying_file.into_local_file_identifier(task);

            Some(
                file_systems
                    .get(&file_system)
                    .ok_or(Error::InvalidIdentifier)?
                    .inner
                    .duplicate(local_file)?
                    .into_unique_file_identifier(file_system)
                    .1,
            )
        } else {
            None
        };

        let new_file = match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => {
                self.pipe_file_system
                    .duplicate(file, underlying_file)
                    .await?
            }
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => {
                let underlying_file = underlying_file.ok_or(Error::InternalError)?;

                self.device_file_system
                    .duplicate(file, underlying_file)
                    .await?
            }
            _ => Self::get_file_system_from_identifier(&file_systems, file_system)?
                .inner
                .duplicate(file)?,
        };

        let (_, new_file) = new_file.into_unique_file_identifier(file_system);

        Ok(new_file)
    }

    pub async fn create_new_task_standard_io(
        &self,
        standard_in: UniqueFileIdentifier,
        standard_error: UniqueFileIdentifier,
        standard_out: UniqueFileIdentifier,
        current_task: TaskIdentifier,
        new_task: TaskIdentifier,
        duplicate: bool,
    ) -> Result<(
        UniqueFileIdentifier,
        UniqueFileIdentifier,
        UniqueFileIdentifier,
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
                Some(FileIdentifier::STANDARD_IN),
            )
            .await?;
        let standard_error = self
            .transfert(
                standard_error,
                current_task,
                new_task,
                Some(FileIdentifier::STANDARD_ERROR),
            )
            .await?;
        let standard_out = self
            .transfert(
                standard_out,
                current_task,
                new_task,
                Some(FileIdentifier::STANDARD_OUT),
            )
            .await?;

        Ok((standard_in, standard_error, standard_out))
    }

    pub async fn is_a_terminal(
        &self,
        file: UniqueFileIdentifier,
        task: TaskIdentifier,
    ) -> Result<bool> {
        let (file_system, file) = file.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::PIPE_FILE_SYSTEM => Err(Error::UnsupportedOperation),
            FileSystemIdentifier::DEVICE_FILE_SYSTEM => {
                self.device_file_system.is_a_terminal(file).await
            }
            _ => Err(Error::UnsupportedOperation),
        }
    }

    pub async fn rename(
        &self,
        old_path: &impl AsRef<Path>,
        new_path: &impl AsRef<Path>,
    ) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (old_file_system_identifier, old_file_system, old_relative_path) =
            Self::get_file_system_from_path(&file_systems, old_path)?; // Get the file system identifier and the relative path

        let (new_file_system_identifier, _, new_relative_path) =
            Self::get_file_system_from_path(&file_systems, new_path)?; // Get the file system identifier and the relative path

        if old_file_system_identifier != new_file_system_identifier {
            return Err(Error::InvalidPath);
        }

        if old_file_system_identifier == new_file_system_identifier {
            old_file_system.rename(old_relative_path, new_relative_path)
        } else {
            Err(Error::UnsupportedOperation) // TODO : Add support for moving between file systems
        }
    }

    pub async fn get_raw_device(&self, path: &impl AsRef<Path>) -> Result<DeviceType> {
        let file_systems = self.file_systems.read().await;

        let (_, file_system, relative_path) = Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let metadata = file_system.get_metadata_from_path(relative_path)?;

        if metadata.get_type() != Kind::BlockDevice && metadata.get_type() != Kind::CharacterDevice
        {
            return Err(Error::UnsupportedOperation);
        }

        if let Some(inode) = metadata.get_inode() {
            self.device_file_system.get_raw_device(inode).await
        } else {
            Err(Error::Corrupted)
        }
    }

    pub async fn get_metadata_from_path(&self, path: &impl AsRef<Path>) -> Result<Metadata> {
        let file_systems = self.file_systems.read().await;

        let (_, file_system, relative_path) = Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        file_system.get_metadata_from_path(relative_path)
    }

    pub async fn get_statistics_from_path(
        &self,
        path: &impl AsRef<Path>,
    ) -> Result<Statistics_type> {
        let file_systems = self.file_systems.read().await;

        let (file_system_identifier, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let metadata = file_system.get_metadata_from_path(relative_path)?;

        Ok(Statistics_type::new(
            file_system_identifier,
            metadata.get_inode().unwrap_or(Inode::new(0)),
            0,
            Size::new(0),
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
        task: TaskIdentifier,
        socket: UniqueFileIdentifier,
        data: &[u8],
    ) -> crate::Result<()> {
        let (file_system, socket) = socket.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => self
                .network_socket_driver
                .ok_or(crate::Error::UnavailableDriver)?
                .send(socket, data)?,
            _ => Err(crate::Error::InvalidFileSystem)?,
        }

        Ok(())
    }

    pub async fn receive(
        &self,
        task: TaskIdentifier,
        socket: UniqueFileIdentifier,
        data: &mut [u8],
    ) -> crate::Result<usize> {
        let (file_system, socket) = socket.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => Ok(self
                .network_socket_driver
                .ok_or(crate::Error::UnavailableDriver)?
                .receive(socket, data)?),
            _ => Err(crate::Error::InvalidFileSystem)?,
        }
    }

    pub async fn send_to(
        &self,
        task: TaskIdentifier,
        socket: UniqueFileIdentifier,
        data: &[u8],
        address: SockerAddress,
    ) -> crate::Result<()> {
        let (file_system, socket) = socket.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => {
                let (ip, port) = address
                    .into_ip_and_port()
                    .ok_or(crate::Error::InvalidParameter)?;

                self.network_socket_driver
                    .ok_or(crate::Error::UnavailableDriver)?
                    .send_to(socket, data, ip, port)?
            }
            _ => Err(crate::Error::InvalidFileSystem)?,
        }

        Ok(())
    }

    pub async fn receive_from(
        &self,
        task: TaskIdentifier,
        socket: UniqueFileIdentifier,
        data: &mut [u8],
    ) -> crate::Result<(usize, SockerAddress)> {
        let (file_system, socket) = socket.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => {
                let (size, ip, port) = self
                    .network_socket_driver
                    .ok_or(crate::Error::UnavailableDriver)?
                    .receive_from(socket, data)?;

                Ok((size, SockerAddress::from_ip_and_port(ip, port)))
            }
            _ => Err(crate::Error::InvalidFileSystem)?,
        }
    }

    fn new_file_identifier(
        &self,
        file_system: FileSystemIdentifier,
        task: TaskIdentifier,
    ) -> crate::Result<LocalFileIdentifier> {
        let iterator = LocalFileIdentifier::get_minimum(task).into_iter();

        match file_system {
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => Ok(self
                .network_socket_driver
                .ok_or(crate::Error::UnavailableDriver)?
                .get_new_socket_identifier(iterator)?
                .ok_or(crate::Error::TooManyOpenFiles)?),
            _ => Err(crate::Error::InvalidFileSystem)?,
        }
    }

    pub async fn bind(
        &self,
        task: TaskIdentifier,
        address: SockerAddress,
        protocol: Protocol,
    ) -> crate::Result<UniqueFileIdentifier> {
        let file_system = match address {
            SockerAddress::IPv4(_, _) | SockerAddress::IPv6(_, _) => {
                FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM
            }
            SockerAddress::Local(_) => {
                todo!()
            }
        };

        let new_socket = self.new_file_identifier(file_system, task)?;

        match file_system {
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => {
                let (ip, port) = if let Some((ip_type, port)) = address.into_ip_and_port() {
                    (ip_type, port)
                } else {
                    unreachable!()
                };

                self.network_socket_driver
                    .ok_or(crate::Error::UnavailableDriver)?
                    .bind(ip, port, protocol, new_socket)?
            }
            _ => return Err(crate::Error::InvalidFileSystem),
        }

        let (_, new_socket) = new_socket.into_unique_file_identifier(file_system);

        Ok(new_socket)
    }

    pub async fn connect(
        &self,
        task: TaskIdentifier,
        address: SockerAddress,
    ) -> crate::Result<UniqueFileIdentifier> {
        let file_system = match address {
            SockerAddress::IPv4(_, _) | SockerAddress::IPv6(_, _) => {
                FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM
            }
            SockerAddress::Local(_) => {
                todo!()
            }
        };

        let new_socket = self.new_file_identifier(file_system, task)?;

        match file_system {
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => {
                let (ip, port) = if let Some((ip_type, port)) = address.into_ip_and_port() {
                    (ip_type, port)
                } else {
                    unreachable!()
                };

                self.network_socket_driver
                    .ok_or(crate::Error::UnavailableDriver)?
                    .connect(ip, port, new_socket)?
            }
            _ => return Err(crate::Error::InvalidFileSystem),
        }

        let (_, new_socket) = new_socket.into_unique_file_identifier(file_system);

        Ok(new_socket)
    }

    pub async fn accept(
        &self,
        task: TaskIdentifier,
        socket: UniqueFileIdentifier,
    ) -> crate::Result<(UniqueFileIdentifier, Option<(IP, Port)>)> {
        let (file_system, socket) = socket.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => {
                let new_socket = self.new_file_identifier(file_system, task)?;

                let (ip, port) = self
                    .network_socket_driver
                    .ok_or(crate::Error::UnavailableDriver)?
                    .accept(socket, new_socket)?;

                let (_, new_socket) = new_socket.into_unique_file_identifier(file_system);

                Ok((new_socket, Some((ip, port))))
            }
            _ => Err(crate::Error::InvalidFileSystem),
        }
    }

    pub async fn set_send_timeout(
        &self,
        task: TaskIdentifier,
        socket: UniqueFileIdentifier,
        timeout: Duration,
    ) -> crate::Result<()> {
        let (file_system, socket) = socket.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => self
                .network_socket_driver
                .ok_or(crate::Error::UnavailableDriver)?
                .set_send_timeout(socket, timeout)?,
            _ => return Err(crate::Error::InvalidFileSystem),
        }

        Ok(())
    }

    pub async fn set_receive_timeout(
        &self,
        task: TaskIdentifier,
        socket: UniqueFileIdentifier,
        timeout: Duration,
    ) -> crate::Result<()> {
        let (file_system, socket) = socket.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => self
                .network_socket_driver
                .ok_or(crate::Error::UnavailableDriver)?
                .set_receive_timeout(socket, timeout)?,
            _ => return Err(crate::Error::InvalidFileSystem),
        }

        Ok(())
    }

    pub async fn get_send_timeout(
        &self,
        task: TaskIdentifier,
        socket: UniqueFileIdentifier,
    ) -> crate::Result<Option<Duration>> {
        let (file_system, socket) = socket.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => Ok(self
                .network_socket_driver
                .ok_or(crate::Error::UnavailableDriver)?
                .get_send_timeout(socket)?),
            _ => Err(crate::Error::InvalidFileSystem),
        }
    }

    pub async fn get_receive_timeout(
        &self,
        task: TaskIdentifier,
        socket: UniqueFileIdentifier,
    ) -> crate::Result<Option<Duration>> {
        let (file_system, socket) = socket.into_local_file_identifier(task);

        match file_system {
            FileSystemIdentifier::NETWORK_SOCKET_FILE_SYSTEM => Ok(self
                .network_socket_driver
                .ok_or(crate::Error::UnavailableDriver)?
                .get_receive_timeout(socket)?),
            _ => Err(crate::Error::InvalidFileSystem),
        }
    }
}

async fn read_line(
    file_system: &dyn FileSystemTraits,
    buffer: &mut String,
    file: LocalFileIdentifier,
    time: Time,
) -> Result<Size> {
    loop {
        let current_buffer = &mut [0; 1];

        let size = file_system.read(file, current_buffer, time)?;

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
    use file_system::LocalFileIdentifier;

    use super::*;

    struct DummyFileSystemType;

    impl FileSystemTraits for DummyFileSystemType {
        fn open(
            &self,
            _: TaskIdentifier,
            _: &Path,
            _: Flags,
            _: Time,
            _: UserIdentifier,
            _: GroupIdentifier,
        ) -> Result<LocalFileIdentifier> {
            todo!()
        }

        fn close(&self, _: LocalFileIdentifier) -> Result<()> {
            todo!()
        }

        fn close_all(&self, _: TaskIdentifier) -> Result<()> {
            todo!()
        }

        fn duplicate(&self, _: LocalFileIdentifier) -> Result<LocalFileIdentifier> {
            todo!()
        }

        fn transfert(
            &self,
            _: TaskIdentifier,
            _: LocalFileIdentifier,
            _: Option<FileIdentifier>,
        ) -> Result<LocalFileIdentifier> {
            todo!()
        }

        fn remove(&self, _: &Path) -> Result<()> {
            todo!()
        }

        fn read(&self, _: LocalFileIdentifier, _: &mut [u8], _: Time) -> Result<Size> {
            todo!()
        }

        fn write(&self, _: LocalFileIdentifier, _: &[u8], _: Time) -> Result<Size> {
            todo!()
        }

        fn rename(&self, _: &Path, _: &Path) -> Result<()> {
            todo!()
        }

        fn set_position(&self, _: LocalFileIdentifier, _: &Position) -> Result<Size> {
            todo!()
        }

        fn flush(&self, _: LocalFileIdentifier) -> Result<()> {
            todo!()
        }

        fn create_directory(
            &self,
            _: &Path,
            _: Time,
            _: UserIdentifier,
            _: GroupIdentifier,
        ) -> Result<()> {
            todo!()
        }

        fn open_directory(&self, _: &Path, _: TaskIdentifier) -> Result<LocalFileIdentifier> {
            todo!()
        }

        fn read_directory(&self, _: LocalFileIdentifier) -> Result<Option<Entry>> {
            todo!()
        }

        fn set_position_directory(&self, _: LocalFileIdentifier, _: Size) -> Result<()> {
            todo!()
        }

        fn get_position_directory(&self, _: LocalFileIdentifier) -> Result<Size> {
            todo!()
        }

        fn rewind_directory(&self, _: LocalFileIdentifier) -> Result<()> {
            todo!()
        }

        fn close_directory(&self, _: LocalFileIdentifier) -> Result<()> {
            todo!()
        }

        fn set_metadata_from_path(&self, _: &Path, _: &Metadata) -> Result<()> {
            todo!()
        }

        fn get_metadata_from_path(&self, _: &Path) -> Result<Metadata> {
            todo!()
        }

        fn get_statistics(&self, _: LocalFileIdentifier) -> Result<Statistics_type> {
            todo!()
        }

        fn get_mode(&self, _: LocalFileIdentifier) -> Result<Mode> {
            todo!()
        }

        fn get_metadata(&self, _: LocalFileIdentifier) -> Result<Metadata> {
            todo!()
        }
    }

    #[test]
    fn test_get_file_system_from_path() {
        let mut file_systems: BTreeMap<FileSystemIdentifier, InternalFileSystemType> =
            BTreeMap::new();

        file_systems.insert(
            1.into(),
            InternalFileSystemType {
                mount_point: PathOwned::new("/".to_string()).unwrap(),
                inner: Box::new(DummyFileSystemType),
            },
        );

        file_systems.insert(
            2.into(),
            InternalFileSystemType {
                mount_point: PathOwned::new("/Foo".to_string()).unwrap(),
                inner: Box::new(DummyFileSystemType),
            },
        );

        file_systems.insert(
            3.into(),
            InternalFileSystemType {
                mount_point: PathOwned::new("/Foo/Bar".to_string()).unwrap(),
                inner: Box::new(DummyFileSystemType),
            },
        );

        let (file_system, _, relative_path) =
            VirtualFileSystemType::get_file_system_from_path(&file_systems, &"/").unwrap();

        assert_eq!(file_system, 1.into());
        assert_eq!(relative_path, Path::ROOT);

        let (file_system, _, relative_path) =
            VirtualFileSystemType::get_file_system_from_path(&file_systems, &"/Foo/Bar").unwrap();

        assert_eq!(file_system, 3.into());
        assert_eq!(relative_path, Path::ROOT);

        let (file_system, _, relative_path) =
            VirtualFileSystemType::get_file_system_from_path(&file_systems, &"/Foo/Bar/Baz")
                .unwrap();

        assert_eq!(file_system, 3.into());
        assert_eq!(relative_path, "/Baz".as_ref());

        let (file_system, _, relative_path) =
            VirtualFileSystemType::get_file_system_from_path(&file_systems, &"/Foo").unwrap();

        assert_eq!(file_system, 2.into());
        assert_eq!(relative_path, Path::ROOT);
    }
}
