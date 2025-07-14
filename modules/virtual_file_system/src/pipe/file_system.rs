use alloc::{collections::BTreeMap, string::String, vec::Vec};

use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use task::TaskIdentifier;

use file_system::{
    get_new_file_identifier, get_new_inode, Error, FileIdentifier, Flags, Inode,
    LocalFileIdentifier, Mode, Result, Size, Status, UniqueFileIdentifier,
};

use super::Pipe;

type OpenPipesInner = (Pipe, Flags, Option<UniqueFileIdentifier>);

struct Inner {
    pub named_pipes: BTreeMap<Inode, Pipe>,
    pub open_pipes: BTreeMap<LocalFileIdentifier, OpenPipesInner>,
}

pub struct FileSystem(RwLock<CriticalSectionRawMutex, Inner>);

impl FileSystem {
    pub fn new() -> Self {
        Self(RwLock::new(Inner {
            named_pipes: BTreeMap::new(),
            open_pipes: BTreeMap::new(),
        }))
    }

    pub async fn get_underlying_file(
        &self,
        file: LocalFileIdentifier,
    ) -> Result<Option<UniqueFileIdentifier>> {
        Ok(self
            .0
            .read()
            .await
            .open_pipes
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?
            .2)
    }

    pub async fn create_unnamed_pipe(
        &self,
        task: TaskIdentifier,
        status: Status,
        buffer_size: usize,
    ) -> Result<(LocalFileIdentifier, LocalFileIdentifier)> {
        let mut inner = self.0.write().await;

        // Create the pipe
        let pipe = Pipe::new(buffer_size);

        // - Create the read file
        let read_flags = Flags::new(Mode::READ_ONLY, None, Some(status));

        let read_file = get_new_file_identifier(task, None, None, &inner.open_pipes)?;

        if inner
            .open_pipes
            .insert(read_file, (pipe.clone(), read_flags, None))
            .is_some()
        {
            return Err(Error::InternalError); // Should never happen
        }

        // - Create the write file
        let write_flags = Flags::new(Mode::WRITE_ONLY, None, Some(status));

        let write_file = get_new_file_identifier(task, None, None, &inner.open_pipes)?;

        if inner
            .open_pipes
            .insert(write_file, (pipe.clone(), write_flags, None))
            .is_some()
        {
            return Err(Error::InternalError); // Should never happen
        }

        Ok((read_file, write_file))
    }

    fn borrow_mutable_inner_2_splited(
        inner: &mut Inner,
    ) -> (
        &mut BTreeMap<Inode, Pipe>,
        &mut BTreeMap<LocalFileIdentifier, OpenPipesInner>,
    ) {
        (&mut inner.named_pipes, &mut inner.open_pipes)
    }

    pub async fn create_named_pipe(&self, buffer_size: usize) -> Result<Inode> {
        let mut inner = self.0.write().await;

        let inode = get_new_inode(&inner.named_pipes)?;

        let pipe = Pipe::new(buffer_size);

        if inner.named_pipes.insert(inode, pipe).is_some() {
            return Err(Error::InternalError); // Should never happen
        }

        Ok(inode)
    }

    pub async fn open(
        &self,
        inode: Inode,
        task: TaskIdentifier,
        flags: Flags,
        underlying_file: UniqueFileIdentifier,
    ) -> Result<LocalFileIdentifier> {
        let mut inner = self.0.write().await;

        let (named_pipes, open_pipes) = Self::borrow_mutable_inner_2_splited(&mut inner);

        let pipe = named_pipes.get(&inode).ok_or(Error::InvalidIdentifier)?;

        let local_file_identifier = get_new_file_identifier(task, None, None, open_pipes)?;

        open_pipes.insert(
            local_file_identifier,
            (pipe.clone(), flags, Some(underlying_file)),
        );

        Ok(local_file_identifier)
    }

    pub async fn close(&self, file: LocalFileIdentifier) -> Result<Option<UniqueFileIdentifier>> {
        let (_, _, underlying_file) = self
            .0
            .write()
            .await
            .open_pipes
            .remove(&file)
            .ok_or(Error::InvalidIdentifier)?;

        Ok(underlying_file)
    }

    pub async fn close_all(&self, task: TaskIdentifier) -> Result<()> {
        let mut inner = self.0.write().await;

        // Get all the keys of the open pipes that belong to the task
        let keys = inner
            .open_pipes
            .keys()
            .filter(|key| key.split().0 == task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the pipes corresponding to the keys
        for key in keys {
            if let Some((pipe, _, _)) = inner.open_pipes.remove(&key) {
                drop(pipe);
            }
        }

        Ok(())
    }

    pub async fn duplicate(
        &self,
        file: LocalFileIdentifier,
        underlying_file: Option<UniqueFileIdentifier>,
    ) -> Result<LocalFileIdentifier> {
        let mut inner = self.0.write().await;

        let (pipe, flags, _) = inner
            .open_pipes
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?
            .clone();

        let new_file = get_new_file_identifier(file.split().0, None, None, &inner.open_pipes)?;

        inner
            .open_pipes
            .insert(new_file, (pipe.clone(), flags, underlying_file));

        Ok(new_file)
    }

    pub async fn transfert(
        &self,
        new_task: TaskIdentifier,
        file: LocalFileIdentifier,
        new_file: Option<FileIdentifier>,
    ) -> Result<LocalFileIdentifier> {
        let mut inner = self.0.write().await;

        let (pipe, flags, underlying_file) = inner
            .open_pipes
            .remove(&file)
            .ok_or(Error::InvalidIdentifier)?;

        let new_file = if let Some(file) = new_file {
            let file = LocalFileIdentifier::new(new_task, file);

            if inner.open_pipes.contains_key(&file) {
                return Err(Error::InvalidIdentifier);
            }

            file
        } else {
            get_new_file_identifier(new_task, None, None, &inner.open_pipes)?
        };

        if inner
            .open_pipes
            .insert(new_file, (pipe, flags, underlying_file))
            .is_some()
        {
            return Err(Error::InternalError); // Should never happen
        }

        Ok(new_file)
    }

    pub async fn remove(&self, inode: Inode) -> Result<()> {
        self.0
            .write()
            .await
            .named_pipes
            .remove(&inode)
            .ok_or(Error::InvalidInode)?;

        Ok(())
    }

    pub async fn read(
        &self,
        file: LocalFileIdentifier,
        buffer: &mut [u8],
    ) -> Result<(Size, Option<UniqueFileIdentifier>)> {
        let inner = self.0.read().await;

        let (pipe, flags, underlying_file) = inner
            .open_pipes
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?;

        if !flags.get_mode().get_read() {
            return Err(Error::InvalidMode);
        }

        if flags.get_status().get_non_blocking() {
            return Ok((pipe.read(buffer).await?, *underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            if let Ok(size) = pipe.read(buffer).await {
                return Ok((size, *underlying_file));
            }
        }
    }

    pub async fn read_line(
        &self,
        file: LocalFileIdentifier,
        buffer: &mut String,
    ) -> Result<(Size, Option<UniqueFileIdentifier>)> {
        let mut inner = self.0.write().await;

        let (pipe, flags, underlying_file) = inner
            .open_pipes
            .get_mut(&file)
            .ok_or(Error::InvalidIdentifier)?;

        if !flags.get_mode().get_read() {
            return Err(Error::InvalidMode);
        }

        if flags.get_status().get_non_blocking() {
            return Ok((pipe.read_line(buffer).await?, *underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            if let Ok(size) = pipe.read_line(buffer).await {
                return Ok((size, *underlying_file));
            }
        }
    }

    pub async fn write(
        &self,
        file: LocalFileIdentifier,
        buffer: &[u8],
    ) -> Result<(Size, Option<UniqueFileIdentifier>)> {
        let inner = self.0.read().await;

        let (pipe, flags, underlying_file) = inner
            .open_pipes
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?;

        if !flags.get_mode().get_write() {
            return Err(Error::InvalidMode);
        }

        if flags.get_status().get_non_blocking() {
            return Ok((pipe.write(buffer).await?, *underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            if let Ok(size) = pipe.write(buffer).await {
                return Ok((size, *underlying_file));
            }
        }
    }

    pub async fn get_mode(&self, file: LocalFileIdentifier) -> Result<Mode> {
        Ok(self
            .0
            .read()
            .await
            .open_pipes
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?
            .1
            .get_mode())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use task::test;

    use super::*;

    #[test]
    async fn test_create_unnamed_pipe() {
        let fs = FileSystem::new();
        let task_id = TaskIdentifier::new(0);
        let status = Status::default();
        let buffer_size = 1024;

        let result = fs.create_unnamed_pipe(task_id, status, buffer_size).await;
        assert!(result.is_ok());

        let (read_file, write_file) = result.unwrap();
        assert!(fs.0.read().await.open_pipes.contains_key(&read_file));
        assert!(fs.0.read().await.open_pipes.contains_key(&write_file));
    }

    #[test]
    async fn test_create_named_pipe() {
        let fs = FileSystem::new();
        let buffer_size = 1024;

        let result = fs.create_named_pipe(buffer_size).await;
        assert!(result.is_ok());

        let inode = result.unwrap();
        assert!(fs.0.read().await.named_pipes.contains_key(&inode));
    }

    #[test]
    async fn test_open_and_close_named_pipe() {
        let fs = FileSystem::new();
        let buffer_size = 1024;
        let task_id = TaskIdentifier::new(0);
        let flags = Flags::new(Mode::READ_WRITE, None, None);

        let inode = fs.create_named_pipe(buffer_size).await.unwrap();
        let file_id = fs
            .open(inode, task_id, flags, UniqueFileIdentifier::from(0))
            .await
            .unwrap();

        assert!(fs.0.read().await.open_pipes.contains_key(&file_id));

        fs.close(file_id).await.unwrap();
        assert!(!fs.0.read().await.open_pipes.contains_key(&file_id));
    }

    #[test]
    async fn test_duplicate_file_identifier() {
        let fs = FileSystem::new();
        let task_id = TaskIdentifier::new(0);
        let status = Status::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();

        let new_file = fs.duplicate(read_file, None).await.unwrap();

        assert!(fs.0.read().await.open_pipes.contains_key(&new_file));
    }

    #[test]
    async fn test_transfert_file_identifier() {
        let fs = FileSystem::new();
        let task_id = TaskIdentifier::new(0);
        let new_task_id = TaskIdentifier::new(1);
        let status = Status::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();
        let new_file = fs.transfert(new_task_id, read_file, None).await.unwrap();

        assert!(fs.0.read().await.open_pipes.contains_key(&new_file));
        assert!(!fs.0.read().await.open_pipes.contains_key(&read_file));
    }

    #[test]
    async fn test_delete_named_pipe() {
        let fs = FileSystem::new();
        let buffer_size = 1024;

        let inode = fs.create_named_pipe(buffer_size).await.unwrap();
        fs.remove(inode).await.unwrap();

        assert!(!fs.0.read().await.named_pipes.contains_key(&inode));
    }

    #[test]
    async fn test_read_and_write_pipe() {
        let fs = FileSystem::new();
        let task_id = TaskIdentifier::new(0);
        let status = Status::default();
        let buffer_size = 1024;

        let (read_file, write_file) = fs
            .create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();

        // Write data to the pipe
        let data = b"hello";
        fs.write(write_file, data).await.unwrap();

        // Read data from the pipe
        let mut buffer = [0; 5];
        fs.read(read_file, &mut buffer).await.unwrap();
        assert_eq!(&buffer, b"hello");
    }

    #[test]
    async fn test_get_mode() {
        let fs = FileSystem::new();
        let task_id = TaskIdentifier::new(0);
        let status = Status::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();
        let mode = fs.get_mode(read_file).await.unwrap();

        assert!(mode.get_read());
        assert!(!mode.get_write());
    }
}
