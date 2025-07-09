use alloc::{collections::BTreeMap, string::String, vec::Vec};

use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use task::Task_identifier_type;

use file_system::{
    get_new_file_identifier, get_new_inode, Error_type, File_identifier_type, Flags_type,
    Inode_type, Local_file_identifier_type, Mode_type, Result_type, Size_type, Status_type,
    Unique_file_identifier_type,
};

use super::Pipe_type;

type Open_pipes_inner_type = (Pipe_type, Flags_type, Option<Unique_file_identifier_type>);

struct Inner_type {
    pub named_pipes: BTreeMap<Inode_type, Pipe_type>,
    pub open_pipes: BTreeMap<Local_file_identifier_type, Open_pipes_inner_type>,
}

pub struct File_system_type(RwLock<CriticalSectionRawMutex, Inner_type>);

impl File_system_type {
    pub fn new() -> Self {
        Self(RwLock::new(Inner_type {
            named_pipes: BTreeMap::new(),
            open_pipes: BTreeMap::new(),
        }))
    }

    pub async fn get_underlying_file(
        &self,
        file: Local_file_identifier_type,
    ) -> Result_type<Option<Unique_file_identifier_type>> {
        Ok(self
            .0
            .read()
            .await
            .open_pipes
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?
            .2)
    }

    pub async fn create_unnamed_pipe(
        &self,
        task: Task_identifier_type,
        status: Status_type,
        buffer_size: usize,
    ) -> Result_type<(Local_file_identifier_type, Local_file_identifier_type)> {
        let mut inner = self.0.write().await;

        // Create the pipe
        let pipe = Pipe_type::new(buffer_size);

        // - Create the read file
        let read_flags = Flags_type::new(Mode_type::READ_ONLY, None, Some(status));

        let read_file = get_new_file_identifier(task, None, None, &inner.open_pipes)?;

        if inner
            .open_pipes
            .insert(read_file, (pipe.clone(), read_flags, None))
            .is_some()
        {
            return Err(Error_type::Internal_error); // Should never happen
        }

        // - Create the write file
        let write_flags = Flags_type::new(Mode_type::WRITE_ONLY, None, Some(status));

        let write_file = get_new_file_identifier(task, None, None, &inner.open_pipes)?;

        if inner
            .open_pipes
            .insert(write_file, (pipe.clone(), write_flags, None))
            .is_some()
        {
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok((read_file, write_file))
    }

    fn borrow_mutable_inner_2_splited(
        inner: &mut Inner_type,
    ) -> (
        &mut BTreeMap<Inode_type, Pipe_type>,
        &mut BTreeMap<Local_file_identifier_type, Open_pipes_inner_type>,
    ) {
        (&mut inner.named_pipes, &mut inner.open_pipes)
    }

    pub async fn create_named_pipe(&self, buffer_size: usize) -> Result_type<Inode_type> {
        let mut inner = self.0.write().await;

        let inode = get_new_inode(&inner.named_pipes)?;

        let pipe = Pipe_type::new(buffer_size);

        if inner.named_pipes.insert(inode, pipe).is_some() {
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok(inode)
    }

    pub async fn open(
        &self,
        inode: Inode_type,
        task: Task_identifier_type,
        flags: Flags_type,
        underlying_file: Unique_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.0.write().await;

        let (named_pipes, open_pipes) = Self::borrow_mutable_inner_2_splited(&mut inner);

        let pipe = named_pipes
            .get(&inode)
            .ok_or(Error_type::Invalid_identifier)?;

        let local_file_identifier = get_new_file_identifier(task, None, None, open_pipes)?;

        open_pipes.insert(
            local_file_identifier,
            (pipe.clone(), flags, Some(underlying_file)),
        );

        Ok(local_file_identifier)
    }

    pub async fn close(
        &self,
        file: Local_file_identifier_type,
    ) -> Result_type<Option<Unique_file_identifier_type>> {
        let (_, _, underlying_file) = self
            .0
            .write()
            .await
            .open_pipes
            .remove(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(underlying_file)
    }

    pub async fn close_all(&self, task: Task_identifier_type) -> Result_type<()> {
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
        file: Local_file_identifier_type,
        underlying_file: Option<Unique_file_identifier_type>,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.0.write().await;

        let (pipe, flags, _) = inner
            .open_pipes
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?
            .clone();

        let new_file = get_new_file_identifier(file.split().0, None, None, &inner.open_pipes)?;

        inner
            .open_pipes
            .insert(new_file, (pipe.clone(), flags, underlying_file));

        Ok(new_file)
    }

    pub async fn transfert(
        &self,
        new_task: Task_identifier_type,
        file: Local_file_identifier_type,
        new_file: Option<File_identifier_type>,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.0.write().await;

        let (pipe, flags, underlying_file) = inner
            .open_pipes
            .remove(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        let new_file = if let Some(file) = new_file {
            let file = Local_file_identifier_type::new(new_task, file);

            if inner.open_pipes.contains_key(&file) {
                return Err(Error_type::Invalid_identifier);
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
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok(new_file)
    }

    pub async fn remove(&self, inode: Inode_type) -> Result_type<()> {
        self.0
            .write()
            .await
            .named_pipes
            .remove(&inode)
            .ok_or(Error_type::Invalid_inode)?;

        Ok(())
    }

    pub async fn read(
        &self,
        file: Local_file_identifier_type,
        buffer: &mut [u8],
    ) -> Result_type<(Size_type, Option<Unique_file_identifier_type>)> {
        let inner = self.0.read().await;

        let (pipe, flags, underlying_file) = inner
            .open_pipes
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        if !flags.get_mode().get_read() {
            return Err(Error_type::Invalid_mode);
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
        file: Local_file_identifier_type,
        buffer: &mut String,
    ) -> Result_type<(Size_type, Option<Unique_file_identifier_type>)> {
        let mut inner = self.0.write().await;

        let (pipe, flags, underlying_file) = inner
            .open_pipes
            .get_mut(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        if !flags.get_mode().get_read() {
            return Err(Error_type::Invalid_mode);
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
        file: Local_file_identifier_type,
        buffer: &[u8],
    ) -> Result_type<(Size_type, Option<Unique_file_identifier_type>)> {
        let inner = self.0.read().await;

        let (pipe, flags, underlying_file) = inner
            .open_pipes
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        if !flags.get_mode().get_write() {
            return Err(Error_type::Invalid_mode);
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

    pub async fn get_mode(&self, file: Local_file_identifier_type) -> Result_type<Mode_type> {
        Ok(self
            .0
            .read()
            .await
            .open_pipes
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?
            .1
            .get_mode())
    }
}

#[cfg(test)]
mod tests {
    use task::Test;

    use super::*;

    #[Test]
    async fn test_create_unnamed_pipe() {
        let fs = File_system_type::new();
        let task_id = Task_identifier_type::new(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let result = fs.create_unnamed_pipe(task_id, status, buffer_size).await;
        assert!(result.is_ok());

        let (read_file, write_file) = result.unwrap();
        assert!(fs.0.read().await.open_pipes.contains_key(&read_file));
        assert!(fs.0.read().await.open_pipes.contains_key(&write_file));
    }

    #[Test]
    async fn test_create_named_pipe() {
        let fs = File_system_type::new();
        let buffer_size = 1024;

        let result = fs.create_named_pipe(buffer_size).await;
        assert!(result.is_ok());

        let inode = result.unwrap();
        assert!(fs.0.read().await.named_pipes.contains_key(&inode));
    }

    #[Test]
    async fn test_open_and_close_named_pipe() {
        let fs = File_system_type::new();
        let buffer_size = 1024;
        let task_id = Task_identifier_type::new(0);
        let flags = Flags_type::new(Mode_type::READ_WRITE, None, None);

        let inode = fs.create_named_pipe(buffer_size).await.unwrap();
        let file_id = fs
            .open(inode, task_id, flags, Unique_file_identifier_type::from(0))
            .await
            .unwrap();

        assert!(fs.0.read().await.open_pipes.contains_key(&file_id));

        fs.close(file_id).await.unwrap();
        assert!(!fs.0.read().await.open_pipes.contains_key(&file_id));
    }

    #[Test]
    async fn test_duplicate_file_identifier() {
        let fs = File_system_type::new();
        let task_id = Task_identifier_type::new(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();

        let new_file = fs.duplicate(read_file, None).await.unwrap();

        assert!(fs.0.read().await.open_pipes.contains_key(&new_file));
    }

    #[Test]
    async fn test_transfert_file_identifier() {
        let fs = File_system_type::new();
        let task_id = Task_identifier_type::new(0);
        let new_task_id = Task_identifier_type::new(1);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();
        let new_file = fs.transfert(new_task_id, read_file, None).await.unwrap();

        assert!(fs.0.read().await.open_pipes.contains_key(&new_file));
        assert!(!fs.0.read().await.open_pipes.contains_key(&read_file));
    }

    #[Test]
    async fn test_delete_named_pipe() {
        let fs = File_system_type::new();
        let buffer_size = 1024;

        let inode = fs.create_named_pipe(buffer_size).await.unwrap();
        fs.remove(inode).await.unwrap();

        assert!(!fs.0.read().await.named_pipes.contains_key(&inode));
    }

    #[Test]
    async fn test_read_and_write_pipe() {
        let fs = File_system_type::new();
        let task_id = Task_identifier_type::new(0);
        let status = Status_type::default();
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

    #[Test]
    async fn test_get_mode() {
        let fs = File_system_type::new();
        let task_id = Task_identifier_type::new(0);
        let status = Status_type::default();
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
