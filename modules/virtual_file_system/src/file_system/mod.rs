mod internal;

use alloc::borrow::ToOwned;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use exported_file_system::{
    AttributeOperations, BaseOperations, BlockDevice, CharacterDevice, DirectoryOperations,
    FileOperations,
};
use synchronization::{
    blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, rwlock::RwLock,
};

use alloc::{boxed::Box, collections::BTreeMap};
use file_system::{
    Attributes, AttributesMask, Context, Entry, FileIdentifier, FileSystemOperations, Flags, Inode,
    Kind, Mode, Path, PathOwned, Permissions, Position, Size, Statistics, Status, Time,
    UniqueFileIdentifier,
};
use network::{IP, Port, Protocol, SocketDriver};
use task::TaskIdentifier;
use time::Duration;
use users::{GroupIdentifier, UserIdentifier};

use crate::pipe::Pipe;
use crate::{Error, Result, SockerAddress};

struct InternalFileSystem {
    pub mount_point: PathOwned,
    pub inner: &'static dyn FileSystemOperations,
}

/// Instance of the virtual file system.
///
/// # Safety
/// I know, it is not safe to use mutable static variables.
/// It is thread safe (after initialization) because it is only read after initialization.
/// It is a pragmatic choice for efficiency in embedded systems contexts (avoid using Arc).
static VIRTUAL_FILE_SYSTEM_INSTANCE: OnceLock<VirtualFileSystem> = OnceLock::new();

pub fn initialize(
    root_file_system: Box<dyn FileSystemOperations>,
    network_socket_driver: Option<&'static dyn SocketDriver>,
) -> Result<&'static VirtualFileSystem<'static>> {
    let virtual_file_system = VirtualFileSystem::new(
        task::get_instance(),
        users::get_instance(),
        time::get_instance(),
        root_file_system,
        network_socket_driver,
    )?;

    Ok(VIRTUAL_FILE_SYSTEM_INSTANCE.get_or_init(|| virtual_file_system))
}

pub fn get_instance() -> &'static VirtualFileSystem<'static> {
    VIRTUAL_FILE_SYSTEM_INSTANCE
        .try_get()
        .expect("Virtual file system not initialized")
}

impl<'a> Item<'a> {
    fn as_base_operations(&self) -> Option<&dyn BaseOperations> {
        match self {
            Item::File(fs) => Some(&**fs),
            Item::BlockDevice(dev) => Some(*dev),
            Item::CharacterDevice(dev) => Some(*dev),
            Item::Pipe(pipe) => Some(&**pipe),
            _ => None,
        }
    }

    fn as_directory_operations(&self) -> Option<&dyn DirectoryOperations> {
        match self {
            Item::Directory(fs) => Some(&**fs),
            _ => None,
        }
    }

    fn as_attributes_operations(&self) -> Option<&dyn AttributeOperations> {
        match self {
            Item::File(fs) => Some(&**fs),
            Item::Directory(fs) => Some(&**fs),
            _ => None,
        }
    }

    fn clone() -> Self {
        match self {
            Item::File(fs) => Item::File(fs.duplicate(&mut Context::new()).unwrap().1),
            Item::Directory(fs) => Item::Directory(fs.clone_box()),
            Item::BlockDevice(dev) => Item::BlockDevice(*dev),
            Item::CharacterDevice(dev) => Item::CharacterDevice(*dev),
            Item::Pipe(pipe) => Item::Pipe(pipe.clone()),
        }
    }
}

struct InternalContext<'a> {
    item: Item<'a>,
    position: Size,
    flags: Flags,
    context: Context,
}

type FileSystemsArray<'a> = Vec<InternalFileSystem>;
type ContextsMap<'a> =
    BTreeMap<UniqueFileIdentifier, RwLock<CriticalSectionRawMutex, InternalContext<'a>>>;
type CharacterDevicesMap<'a> = BTreeMap<Inode, &'a dyn CharacterDevice>;
type BlockDevicesMap<'a> = BTreeMap<Inode, &'a dyn BlockDevice>;
type PipeMap = BTreeMap<Inode, Rc<Pipe>>;

/// The virtual file system.
///
/// It is a singleton.
pub struct VirtualFileSystem<'a> {
    /// Mounted file systems.
    file_systems: RwLock<CriticalSectionRawMutex, FileSystemsArray<'a>>,
    /// Character devices.
    character_device: RwLock<CriticalSectionRawMutex, CharacterDevicesMap<'a>>,
    /// Block devices.
    block_device: RwLock<CriticalSectionRawMutex, BlockDevicesMap<'a>>,
    /// Pipes.
    named_pipes: RwLock<CriticalSectionRawMutex, PipeMap>,
    /// Network sockets.
    network_socket_driver: Option<&'a dyn SocketDriver>,
}

unsafe impl Send for VirtualFileSystem<'_> {}
unsafe impl Sync for VirtualFileSystem<'_> {}

impl<'a> VirtualFileSystem<'a> {
    pub const STANDARD_INPUT_FILE_IDENTIFIER: FileIdentifier = FileIdentifier::new(0);
    pub const STANDARD_OUTPUT_FILE_IDENTIFIER: FileIdentifier = FileIdentifier::new(1);
    pub const STANDARD_ERROR_FILE_IDENTIFIER: FileIdentifier = FileIdentifier::new(2);

    pub fn new(
        _: &'static task::Manager,
        _: &'static users::Manager,
        _: &'static time::Manager,
        root_file_system: Box<dyn FileSystemOperations>,
        network_socket_driver: Option<&'a dyn SocketDriver>,
    ) -> Result<Self> {
        let mut file_systems = Vec::new();

        let file_system = Box::leak(root_file_system);

        file_systems.push(InternalFileSystem {
            mount_point: PathOwned::new("/".to_string()).unwrap(),
            inner: file_system,
        });

        Ok(Self {
            file_systems: RwLock::new(file_systems),
            character_device: RwLock::new(BTreeMap::new()),
            block_device: RwLock::new(BTreeMap::new()),
            named_pipes: RwLock::new(BTreeMap::new()),
            network_socket_driver,
        })
    }

    pub async fn uninitialize(&self) {
        let mut file_systems = self.file_systems.write().await;

        for internal_file_system in file_systems.drain(..) {
            let _: Box<dyn FileSystemOperations> =
                unsafe { Box::from_raw(internal_file_system.inner as *const _ as *mut _) };
        }
    }

    fn get_new_inode<T>(devices: &BTreeMap<Inode, T>) -> Option<Inode> {
        for inode in 1..Inode::MAX {
            if !devices.contains_key(&inode) {
                return Some(inode);
            }
        }

        None
    }

    /// Mount a file system at a given mount point.
    pub async fn mount_file_system(
        &'static self,
        file_system: impl FileSystemOperations + 'static,
        path: impl AsRef<Path>,
        task: TaskIdentifier,
    ) -> Result<()> {
        let path = path.as_ref();

        if !path.is_valid() {
            return Err(Error::InvalidPath);
        }

        if !path.is_absolute() {
            return Err(Error::InvalidPath);
        }

        let mut file_systems = self.file_systems.write().await; // Get the file systems

        // Create a directory in the underlying file system
        let (parent_file_system, relative_path, _) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let (time, user, group) = self.get_time_user_group(task).await?;

        parent_file_system.create_directory(relative_path)?;
        let attributes = Attributes::new()
            .set_user(user)
            .set_group(group)
            .set_creation(time)
            .set_modification(time)
            .set_access(time)
            .set_kind(Kind::Directory);
        FileSystemOperations::set_attributes(parent_file_system, relative_path, &attributes)?;

        let file_system = Box::leak(Box::new(file_system));

        file_systems.push(InternalFileSystem {
            mount_point: path.to_owned(),
            inner: file_system,
        });

        Ok(())
    }

    pub async fn unmount_file_system(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Box<dyn FileSystemOperations>> {
        let path = path.as_ref();

        if !path.is_valid() || !path.is_absolute() {
            return Err(Error::InvalidPath);
        }

        let mut file_systems = self.file_systems.write().await; // Get the file systems

        let (file_system, relative_path, i) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        // Cannot unmount the root file system
        if !relative_path.is_root() {
            return Err(Error::InvalidPath);
        }

        // Check if the file system is in use
        let contexts = self.contexts.write().await;
        for (_, context) in contexts.iter() {
            let context = context.read().await;

            // if let Some(fs) = context.item.as_file_system_operations() {
            //     if fs as *const _ == file_system as *const _ {
            //         return Err(Error::RessourceBusy);
            //     }
            // }
        }

        // Check if there are mounted devices

        // Finally unmount the file system
        let internal_file_system = file_systems.remove(i);

        Ok(unsafe { Box::from_raw(internal_file_system.inner as *const _ as *mut _) })
    }

    fn get_file_system_from_path<'b, 'c>(
        file_systems: &'b Vec<InternalFileSystem>,
        path: &'c impl AsRef<Path>,
    ) -> Result<(&'b dyn FileSystemOperations, &'c Path, usize)> {
        let mut result_score = 0;
        let mut result: Option<(&InternalFileSystem, usize)> = None;

        let path = path.as_ref();
        let path_components = path.get_components();

        for (i, file_system) in file_systems.iter().enumerate() {
            let mount_point: &Path = file_system.mount_point.as_ref();
            let mount_point_components = mount_point.get_components();

            let score = path_components
                .clone()
                .get_common_components(mount_point_components);

            if result_score < score {
                result_score = score;
                result = Some((file_system, i));
            }
        }

        let internal_file_system = result.ok_or(Error::InvalidPath)?;

        let relative_path = path
            .strip_prefix_absolute(internal_file_system.0.mount_point.as_ref())
            .ok_or(Error::InvalidPath)?;

        Ok((
            internal_file_system.0.inner,
            relative_path,
            internal_file_system.1,
        ))
    }

    pub async fn open(
        &self,
        path: &impl AsRef<Path>,
        flags: Flags,
        task: TaskIdentifier,
    ) -> Result<UniqueFileIdentifier> {
        let path = path.as_ref();

        let file_systems = self.file_systems.read().await; // Get the file systems

        let (file_system, relative_path, i) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let time: Time = time::get_instance().get_current_time()?.into();

        let user = task::get_instance().get_user(task).await?;
        let group = users::get_instance().get_user_primary_group(user).await?;

        let (mode, open, status) = flags.split();

        if open.get_create() {
            match file_system.create_file(relative_path) {
                Ok(()) => {
                    let attributes = Attributes::new()
                        .set_user(user)
                        .set_group(group)
                        .set_creation(time)
                        .set_modification(time)
                        .set_kind(Kind::File)
                        .set_access(time);
                    file_system.set_attributes(path, &attributes);
                }
                Err(file_system::Error::AlreadyExists) => {
                    if open.get_exclusive() {
                        return Err(Error::AlreadyExists);
                    }
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        let attributes = if mode.get_write() {
            Attributes::new().set_modification(time).set_access(time)
        } else {
            Attributes::new().set_access(time)
        };
        file_system.set_attributes(path, &attributes);

        let mut attributes = Attributes::new().set_mask(
            AttributesMask::INODE
                | AttributesMask::KIND
                | AttributesMask::USER
                | AttributesMask::GROUP
                | AttributesMask::PERMISSIONS,
        );
        file_system.get_attributes(relative_path, &mut attributes)?;

        let kind = attributes.get_kind().ok_or(Error::MissingAttribute)?;
        let owner_user = *attributes.get_user().ok_or(Error::MissingAttribute)?;
        let owner_group = *attributes.get_group().ok_or(Error::MissingAttribute)?;
        let permissions = *attributes
            .get_permissions()
            .ok_or(Error::MissingAttribute)?;
        let inode = *attributes.get_inode().ok_or(Error::MissingAttribute)?;

        let has_permissions = Self::has_permissions(
            &users::get_instance(),
            user,
            mode.into_permission(),
            owner_user,
            owner_group,
            permissions,
        )
        .await;

        if !has_permissions {
            return Err(Error::PermissionDenied);
        }

        let mut context = Context::new();

        let context = match kind {
            Kind::CharacterDevice => {
                let character_device = *self
                    .character_device
                    .read()
                    .await
                    .get(&inode)
                    .ok_or(Error::InvalidIdentifier)?;

                character_device.open(&mut context);

                InternalContext {
                    item: Item::CharacterDevice(character_device),
                    position: 0,
                    flags,
                    context,
                }
            }
            Kind::BlockDevice => {
                let block_device = *self
                    .block_device
                    .read()
                    .await
                    .get(&inode)
                    .ok_or(Error::InvalidIdentifier)?;

                InternalContext {
                    item: Item::BlockDevice(block_device),
                    position: 0,
                    flags,
                    context,
                }
            }
            Kind::Pipe => {
                let pipe = *self
                    .named_pipes
                    .read()
                    .await
                    .get(&inode)
                    .ok_or(Error::InvalidIdentifier)?;

                InternalContext {
                    item: Item::Pipe(pipe.clone()),
                    position: 0,
                    flags,
                    context,
                }
            }
            Kind::File => {
                let file = file_system.lookup_file(&mut context, relative_path, &flags)?;

                InternalContext {
                    item: Item::File(file),
                    position: 0,
                    flags,
                    context,
                }
            }
            Kind::Directory => {
                if flags.get_mode() != Mode::READ_ONLY {
                    return Err(Error::InvalidMode);
                }

                if open.get_create() || open.get_exclusive() || open.get_truncate() {
                    return Err(Error::InvalidParameter);
                }

                let directory = file_system.lookup_directory(&mut context, relative_path)?;

                InternalContext {
                    item: Item::Directory(directory),
                    position: 0,
                    flags,
                    context,
                }
            }
            _ => {
                todo!()
            }
        };

        let unique_file = self.add_context(task, None, RwLock::new(context)).await?;

        Ok(unique_file)
    }

    pub async fn close(&self, file: UniqueFileIdentifier) -> crate::Result<()> {
        Self::close_internal(&mut self.contexts.write().await, file).await
    }

    pub async fn read(&self, file: UniqueFileIdentifier, buffer: &mut [u8]) -> Result<usize> {
        self.perform_file_operation(file, Mode::READ_ONLY, |device, context| {
            let read_size = device.read(&mut context.context, buffer, context.position)?;

            context.position += read_size as Size;

            Ok(read_size)
        })
        .await
    }

    pub async fn read_until(
        &self,
        file: UniqueFileIdentifier,
        buffer: &mut [u8],
        delimiter: u8,
    ) -> Result<usize> {
        self.perform_file_operation(file, Mode::READ_ONLY, |device, context| {
            let read_size =
                device.read_until(&mut context.context, buffer, context.position, delimiter)?;

            context.position += read_size as Size;

            Ok(read_size)
        })
        .await
    }

    pub async fn write(&self, file: UniqueFileIdentifier, buffer: &[u8]) -> Result<usize> {
        self.perform_file_operation(file, Mode::WRITE_ONLY, |device, context| {
            let write_size = device.write(&mut context.context, buffer, context.position)?;

            context.position += write_size as Size;

            Ok(write_size)
        })
        .await
    }

    pub async fn set_position(
        &self,
        file: UniqueFileIdentifier,
        position: &Position,
    ) -> Result<Size> {
        self.perform_file_operation(file, Mode::NONE, |device, context| {
            let new_position = match position {
                Position::Start(offset) => *offset,
                Position::Current(offset) => {
                    if *offset < 0 {
                        context.position.saturating_sub((-*offset) as Size)
                    } else {
                        context.position.saturating_add(*offset as Size)
                    }
                }
                Position::End(offset) => {
                    let size = context
                        .item
                        .as_base_operations()
                        .ok_or(Error::UnsupportedOperation)?
                        .get_size(&mut context.context)?;

                    if *offset < 0 {
                        size.saturating_sub((-*offset) as Size)
                    } else {
                        size.saturating_add(*offset as Size)
                    }
                }
            };

            context.position = new_position;

            device.set_position(&mut context.context, position)?;

            Ok(new_position)
        })
        .await
    }

    pub async fn set_owner(
        &self,
        path: impl AsRef<Path>,
        user: Option<UserIdentifier>,
        group: Option<GroupIdentifier>,
    ) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (file_system, _, _) = Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let mut attributes = Attributes::new();

        if let Some(user) = user {
            attributes = attributes.set_user(user);
        }

        if let Some(group) = group {
            attributes = attributes.set_group(group);
        }

        Ok(file_system.set_attributes(path.as_ref(), &attributes)?)
    }

    pub async fn set_permissions(
        &self,
        path: impl AsRef<Path>,
        permissions: Permissions,
    ) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (file_system, relative_path, _) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let attributes = Attributes::new().set_permissions(permissions);

        Ok(file_system.set_attributes(relative_path, &attributes)?)
    }

    pub async fn close_all(&self, task_identifier: TaskIdentifier) -> Result<()> {
        let mut contexts = self.contexts.write().await;

        for (file_identifier, context) in contexts.iter_mut() {
            if file_identifier.split().0 == task_identifier {
                let mut context = context.write().await;

                if let Some(directory) = context.item.as_directory_operations() {
                    directory.close(&mut context.context)?;
                } else if let Some(file) = context.item.as_base_operations() {
                    file.close(&mut context.context)?;
                }
            }
        }

        Ok(())
    }

    pub async fn mount_block_device(
        &'static self,
        task: TaskIdentifier,
        path: impl AsRef<Path>,
        device: &'static dyn BlockDevice,
    ) -> Result<()> {
        let path = path.as_ref();

        let file_systems = self.file_systems.read().await; // Get the file systems

        let mut block_devices = self.block_device.write().await;

        let (file_system, relative_path, _) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let time: Time = time::get_instance().get_current_time()?.into();

        let user = task::get_instance().get_user(task).await?;
        let group = users::get_instance().get_user_primary_group(user).await?;

        file_system.create_file(relative_path)?;
        let attributes = Attributes::new()
            .set_user(user)
            .set_group(group)
            .set_creation(time)
            .set_modification(time)
            .set_access(time)
            .set_kind(Kind::BlockDevice);
        file_system.set_attributes(path, &attributes);

        let inode = Self::get_new_inode(&block_devices).ok_or(Error::TooManyInodes)?;

        block_devices.insert(inode, device);

        Ok(())
    }

    pub async fn mount_character_device(
        &self,
        task: TaskIdentifier,
        path: &impl AsRef<Path>,
        device: &dyn CharacterDevice,
    ) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let character_devices = self.character_device.write().await;

        let (file_system, relative_path, _) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let (time, user, group) = self.get_time_user_group(task).await?;

        let inode = Self::get_new_inode(&character_devices).ok_or(Error::TooManyInodes)?;
        character_devices.insert(inode, device);

        file_system.create_file(relative_path)?;
        let attributes = Attributes::new()
            .set_user(user)
            .set_group(group)
            .set_creation(time)
            .set_modification(time)
            .set_access(time)
            .set_kind(Kind::CharacterDevice)
            .set_inode(inode);
        file_system.set_attributes(path.as_ref(), &attributes)?;

        Ok(())
    }

    pub async fn create_named_pipe(
        &self,
        path: &impl AsRef<Path>,
        size: usize,
        task: TaskIdentifier,
    ) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (file_system, relative_path, _) = Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let (time, user, group) = self.get_time_user_group(task).await?;

        let inode = self.named_pipes.create_named_pipe(size).await?;

        let time: Time = time::get_instance().get_current_time()?.into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        let mut metadata = Attributes::get_default(Kind::Pipe, time, user, group)
            .ok_or(Error::InvalidParameter)?;
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
        let pipe = Rc::new(Pipe::new(size));

        let read_context = InternalContext {
            item: Item::Pipe(pipe.clone()),
            position: 0,
            flags: Mode::READ_ONLY.into(),
            context: Context::new(),
        };
        let write_context = InternalContext {
            item: Item::Pipe(pipe.clone()),
            position: 0,
            flags: Mode::WRITE_ONLY.into(),
            context: Context::new(),
        };

        let read_identifier = self
            .add_context(task, None, RwLock::new(read_context))
            .await?;

        let write_identifier = self
            .add_context(task, None, RwLock::new(write_context))
            .await?;

        Ok((read_identifier, write_identifier))
    }

    pub async fn remove(&self, path: impl AsRef<Path>) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (file_system, relative_path, _) =
            Self::get_file_system_from_path(&file_systems, &path)?; // Get the file system identifier and the relative path

        let mut attributes =
            Attributes::new().set_mask(AttributesMask::KIND | AttributesMask::INODE);

        file_system.get_attributes(&path.as_ref(), &mut attributes)?;
        let inode = *attributes.get_inode().ok_or(Error::MissingAttribute)?;
        let kind = attributes.get_kind().ok_or(Error::MissingAttribute)?;

        match kind {
            Kind::Directory => {
                return Err(Error::UnsupportedOperation);
            }
            Kind::Pipe => {
                let mut named_pipes = self.named_pipes.write().await;

                named_pipes.remove(&inode);
            }
            _ => {}
        }

        Ok(file_system.remove(relative_path)?)
    }

    pub async fn transfer(
        &self,
        file: UniqueFileIdentifier,
        new_task: TaskIdentifier,
        new_file: Option<FileIdentifier>,
    ) -> Result<UniqueFileIdentifier> {
        let mut contexts = self.contexts.write().await;

        let context = contexts.remove(&file).ok_or(Error::InvalidIdentifier)?;

        self.add_context(new_task, new_file, context).await
    }

    pub async fn flush(&self, file: UniqueFileIdentifier) -> Result<()> {
        self.perform_file_operation(file, Mode::READ_ONLY, |device, context| {
            Ok(device.flush(&mut context.context)?)
        })
        .await
    }

    pub async fn get_statistics(&self, file: UniqueFileIdentifier) -> Result<Statistics> {
        let contexts = self.contexts.read().await;

        let context = contexts
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?
            .read()
            .await;

        match context.item.as_attributes_operations() {
            Some(device) => device.get_statistics(&mut context.context),
            None => Err(Error::UnsupportedOperation),
        }
    }

    pub async fn read_directory(&self, file: UniqueFileIdentifier) -> Result<Option<Entry>> {
        self.perform_directory_operation(file, Mode::READ_ONLY, |directory, context| {
            let entry = directory.read(&mut context.context)?;

            if entry.is_some() {
                context.position += 1;
            }

            Ok(entry)
        })
        .await
    }

    pub async fn set_position_directory(
        &self,
        file: UniqueFileIdentifier,
        position: Size,
    ) -> Result<()> {
        self.perform_directory_operation(file, Mode::NONE, |directory, context| {
            directory.set_position(&mut context.context, position)?;

            context.position = position;

            Ok(())
        })
        .await
    }

    pub async fn get_position_directory(&self, file: UniqueFileIdentifier) -> Result<Size> {
        self.perform_directory_operation(file, Mode::READ_ONLY, |directory, context| {
            let position = directory.get_position(&mut context.context)?;

            Ok(position)
        })
        .await
    }

    pub async fn rewind_directory(&self, file: UniqueFileIdentifier) -> Result<()> {
        self.perform_directory_operation(file, Mode::READ_ONLY, |directory, context| {
            directory.rewind(&mut context.context)?;

            context.position = 0;

            Ok(())
        })
        .await
    }

    pub async fn create_directory(
        &self,
        path: &impl AsRef<Path>,
        task: TaskIdentifier,
    ) -> Result<()> {
        let file_systems = self.file_systems.read().await; // Get the file systems

        let (file_system, relative_path, _) = Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let (time, user, group) = self.get_time_user_group(task).await?;

        file_system.create_directory(relative_path)?;

        let attributes = Attributes::new()
            .set_user(user)
            .set_group(group)
            .set_creation(time)
            .set_modification(time)
            .set_access(time)
            .set_kind(Kind::Directory);
        file_system.set_attributes(path.as_ref(), &attributes);

        Ok(())
    }

    pub async fn get_mode(&self, file: UniqueFileIdentifier) -> Result<Mode> {
        let contexts = self.contexts.read().await;

        let context = contexts
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?
            .read()
            .await;

        Ok(context.flags.get_mode())
    }

    pub async fn duplicate(
        &self,
        file: UniqueFileIdentifier,
        task: TaskIdentifier,
    ) -> Result<UniqueFileIdentifier> {
        let mut contexts = self.contexts.write().await;

        let current_context = contexts
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?
            .read()
            .await;

        let mut new_context = Context::new();

        current_context
            .item
            .as_base_operations()
            .ok_or(Error::UnsupportedOperation)?
            .duplicate(&mut current_context.context, &mut new_context)?;

        self.add_context(
            task,
            None,
            RwLock::new(InternalContext {
                item: current_context.item.clone(),
                position: current_context.position,
                flags: current_context.flags,
                context: new_context,
            }),
        )
        .await
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

    pub async fn get_raw_device(&self, path: &impl AsRef<Path>) -> Result<Device> {
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

    pub async fn get_statistics_from_path(&self, path: &impl AsRef<Path>) -> Result<Statistics> {
        let file_systems = self.file_systems.read().await;

        let (file_system_identifier, file_system, relative_path) =
            Self::get_file_system_from_path(&file_systems, path)?; // Get the file system identifier and the relative path

        let metadata = file_system.get_metadata_from_path(relative_path)?;

        Ok(Statistics::new(
            file_system_identifier,
            metadata.get_inode().unwrap_or(Inode::new(0)),
            0,
            0,
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
        todo!()
    }

    pub async fn receive(
        &self,
        task: TaskIdentifier,
        socket: UniqueFileIdentifier,
        data: &mut [u8],
    ) -> crate::Result<usize> {
        todo!()
    }

    pub async fn send_to(
        &self,
        task: TaskIdentifier,
        socket: UniqueFileIdentifier,
        data: &[u8],
        address: SockerAddress,
    ) -> crate::Result<()> {
        todo!()
    }

    pub async fn receive_from(
        &self,
        socket: UniqueFileIdentifier,
        data: &mut [u8],
    ) -> crate::Result<(usize, SockerAddress)> {
        todo!()
    }

    pub async fn bind(
        &self,
        address: SockerAddress,
        protocol: Protocol,
    ) -> crate::Result<UniqueFileIdentifier> {
        todo!()
    }

    pub async fn connect(&self, address: SockerAddress) -> crate::Result<UniqueFileIdentifier> {
        todo!()
    }

    pub async fn accept(
        &self,
        socket: UniqueFileIdentifier,
    ) -> crate::Result<(UniqueFileIdentifier, Option<(IP, Port)>)> {
        todo!()
    }

    pub async fn set_send_timeout(
        &self,
        socket: UniqueFileIdentifier,
        timeout: Duration,
    ) -> crate::Result<()> {
        todo!()
    }

    pub async fn set_receive_timeout(
        &self,
        socket: UniqueFileIdentifier,
        timeout: Duration,
    ) -> crate::Result<()> {
        todo!()
    }

    pub async fn get_send_timeout(
        &self,
        socket: UniqueFileIdentifier,
    ) -> crate::Result<Option<Duration>> {
        todo!()
    }

    pub async fn get_receive_timeout(
        &self,
        socket: UniqueFileIdentifier,
    ) -> crate::Result<Option<Duration>> {
        todo!()
    }
}

async fn read_line(
    file_system: &dyn FileSystemOperations,
    buffer: &mut String,
    file: UniqueFileIdentifier,
    time: Time,
) -> Result<Size> {
    loop {
        let current_buffer = &mut [0; 1];

        let size = file_system.read(file, current_buffer, time)?;

        if size == 0 {
            task::Manager::sleep(core::time::Duration::from_millis(10)).await;
            // yield_now().await; // Yield to allow other tasks to run, especially in a blocking operation
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
    use file_system::UniqueFileIdentifier;

    use super::*;

    struct DummyFileSystem;

    impl FileSystemOperations for DummyFileSystem {
        fn open(
            &self,
            _: TaskIdentifier,
            _: &Path,
            _: Flags,
            _: Time,
            _: UserIdentifier,
            _: GroupIdentifier,
        ) -> Result<UniqueFileIdentifier> {
            todo!()
        }

        fn close(&self, _: UniqueFileIdentifier) -> Result<()> {
            todo!()
        }

        fn close_all(&self, _: TaskIdentifier) -> Result<()> {
            todo!()
        }

        fn duplicate(&self, _: UniqueFileIdentifier) -> Result<UniqueFileIdentifier> {
            todo!()
        }

        fn transfer(
            &self,
            _: TaskIdentifier,
            _: UniqueFileIdentifier,
            _: Option<FileIdentifier>,
        ) -> Result<UniqueFileIdentifier> {
            todo!()
        }

        fn remove(&self, _: &Path) -> Result<()> {
            todo!()
        }

        fn read(&self, _: UniqueFileIdentifier, _: &mut [u8], _: Time) -> Result<Size> {
            todo!()
        }

        fn write(&self, _: UniqueFileIdentifier, _: &[u8], _: Time) -> Result<Size> {
            todo!()
        }

        fn rename(&self, _: &Path, _: &Path) -> Result<()> {
            todo!()
        }

        fn set_position(&self, _: UniqueFileIdentifier, _: &Position) -> Result<Size> {
            todo!()
        }

        fn flush(&self, _: UniqueFileIdentifier) -> Result<()> {
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

        fn open_directory(&self, _: &Path, _: TaskIdentifier) -> Result<UniqueFileIdentifier> {
            todo!()
        }

        fn read_directory(&self, _: UniqueFileIdentifier) -> Result<Option<Entry>> {
            todo!()
        }

        fn set_position_directory(&self, _: UniqueFileIdentifier, _: Size) -> Result<()> {
            todo!()
        }

        fn get_position_directory(&self, _: UniqueFileIdentifier) -> Result<Size> {
            todo!()
        }

        fn rewind_directory(&self, _: UniqueFileIdentifier) -> Result<()> {
            todo!()
        }

        fn close_directory(&self, _: UniqueFileIdentifier) -> Result<()> {
            todo!()
        }

        fn set_metadata_from_path(&self, _: &Path, _: &Attributes) -> Result<()> {
            todo!()
        }

        fn get_metadata_from_path(&self, _: &Path) -> Result<Attributes> {
            todo!()
        }

        fn get_attributes(&self, _: UniqueFileIdentifier) -> Result<Statistics> {
            todo!()
        }

        fn get_metadata(&self, _: UniqueFileIdentifier) -> Result<Attributes> {
            todo!()
        }
    }

    #[test]
    fn test_get_file_system_from_path() {
        let mut file_systems: BTreeMap<FileSystemIdentifier, InternalFileSystem> = BTreeMap::new();

        file_systems.insert(
            1.into(),
            InternalFileSystem {
                mount_point: PathOwned::new("/".to_string()).unwrap(),
                inner: Box::new(DummyFileSystem),
            },
        );

        file_systems.insert(
            2.into(),
            InternalFileSystem {
                mount_point: PathOwned::new("/Foo".to_string()).unwrap(),
                inner: Box::new(DummyFileSystem),
            },
        );

        file_systems.insert(
            3.into(),
            InternalFileSystem {
                mount_point: PathOwned::new("/Foo/Bar".to_string()).unwrap(),
                inner: Box::new(DummyFileSystem),
            },
        );

        let (file_system, _, relative_path) =
            VirtualFileSystem::get_file_system_from_path(&file_systems, &"/").unwrap();

        assert_eq!(file_system, 1.into());
        assert_eq!(relative_path, Path::ROOT);

        let (file_system, _, relative_path) =
            VirtualFileSystem::get_file_system_from_path(&file_systems, &"/Foo/Bar").unwrap();

        assert_eq!(file_system, 3.into());
        assert_eq!(relative_path, Path::ROOT);

        let (file_system, _, relative_path) =
            VirtualFileSystem::get_file_system_from_path(&file_systems, &"/Foo/Bar/Baz").unwrap();

        assert_eq!(file_system, 3.into());
        assert_eq!(relative_path, "/Baz".as_ref());

        let (file_system, _, relative_path) =
            VirtualFileSystem::get_file_system_from_path(&file_systems, &"/Foo").unwrap();

        assert_eq!(file_system, 2.into());
        assert_eq!(relative_path, Path::ROOT);
    }
}
