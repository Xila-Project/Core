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

    pub async fn Create_unnamed_pipe(
        &self,
        task: Task_identifier_type,
        status: Status_type,
        buffer_size: usize,
    ) -> Result_type<(Local_file_identifier_type, Local_file_identifier_type)> {
        let mut inner = self.0.write().await;

        // Create the pipe
        let Pipe = Pipe_type::New(buffer_size);

        // - Create the read file
        let Read_flags = Flags_type::New(Mode_type::READ_ONLY, None, Some(status));

        let Read_file = get_new_file_identifier(task, None, None, &inner.open_pipes)?;

        if inner
            .open_pipes
            .insert(Read_file, (Pipe.clone(), Read_flags, None))
            .is_some()
        {
            return Err(Error_type::Internal_error); // Should never happen
        }

        // - Create the write file
        let Write_flags = Flags_type::New(Mode_type::WRITE_ONLY, None, Some(status));

        let Write_file = get_new_file_identifier(task, None, None, &inner.open_pipes)?;

        if inner
            .open_pipes
            .insert(Write_file, (Pipe.clone(), Write_flags, None))
            .is_some()
        {
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok((Read_file, Write_file))
    }

    fn Borrow_mutable_inner_2_splited(
        inner: &mut Inner_type,
    ) -> (
        &mut BTreeMap<Inode_type, Pipe_type>,
        &mut BTreeMap<Local_file_identifier_type, Open_pipes_inner_type>,
    ) {
        (&mut inner.named_pipes, &mut inner.open_pipes)
    }

    pub async fn Create_named_pipe(&self, Buffer_size: usize) -> Result_type<Inode_type> {
        let mut inner = self.0.write().await;

        let Inode = get_new_inode(&inner.named_pipes)?;

        let Pipe = Pipe_type::New(Buffer_size);

        if inner.named_pipes.insert(Inode, Pipe).is_some() {
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok(Inode)
    }

    pub async fn Open(
        &self,
        inode: Inode_type,
        task: Task_identifier_type,
        flags: Flags_type,
        underlying_file: Unique_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.0.write().await;

        let (Named_pipes, Open_pipes) = Self::Borrow_mutable_inner_2_splited(&mut inner);

        let Pipe = Named_pipes
            .get(&inode)
            .ok_or(Error_type::Invalid_identifier)?;

        let Local_file_identifier = get_new_file_identifier(task, None, None, Open_pipes)?;

        Open_pipes.insert(
            Local_file_identifier,
            (Pipe.clone(), flags, Some(underlying_file)),
        );

        Ok(Local_file_identifier)
    }

    pub async fn Close(
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

    pub async fn Close_all(&self, Task: Task_identifier_type) -> Result_type<()> {
        let mut inner = self.0.write().await;

        // Get all the keys of the open pipes that belong to the task
        let Keys = inner
            .open_pipes
            .keys()
            .filter(|Key| Key.Split().0 == Task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the pipes corresponding to the keys
        for Key in Keys {
            if let Some((pipe, _, _)) = inner.open_pipes.remove(&Key) {
                drop(pipe);
            }
        }

        Ok(())
    }

    pub async fn Duplicate(
        &self,
        file: Local_file_identifier_type,
        underlying_file: Option<Unique_file_identifier_type>,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.0.write().await;

        let (Pipe, Flags, _) = inner
            .open_pipes
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?
            .clone();

        let New_file = get_new_file_identifier(file.Split().0, None, None, &inner.open_pipes)?;

        inner
            .open_pipes
            .insert(New_file, (Pipe.clone(), Flags, underlying_file));

        Ok(New_file)
    }

    pub async fn Transfert(
        &self,
        new_task: Task_identifier_type,
        file: Local_file_identifier_type,
        new_file: Option<File_identifier_type>,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.0.write().await;

        let (Pipe, Flags, Underlying_file) = inner
            .open_pipes
            .remove(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        let New_file = if let Some(File) = new_file {
            let file = Local_file_identifier_type::New(new_task, File);

            if inner.open_pipes.contains_key(&file) {
                return Err(Error_type::Invalid_identifier);
            }

            file
        } else {
            get_new_file_identifier(new_task, None, None, &inner.open_pipes)?
        };

        if inner
            .open_pipes
            .insert(New_file, (Pipe, Flags, Underlying_file))
            .is_some()
        {
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok(New_file)
    }

    pub async fn Remove(&self, Inode: Inode_type) -> Result_type<()> {
        self.0
            .write()
            .await
            .named_pipes
            .remove(&Inode)
            .ok_or(Error_type::Invalid_inode)?;

        Ok(())
    }

    pub async fn Read(
        &self,
        file: Local_file_identifier_type,
        buffer: &mut [u8],
    ) -> Result_type<(Size_type, Option<Unique_file_identifier_type>)> {
        let inner = self.0.read().await;

        let (Pipe, Flags, Underlying_file) = inner
            .open_pipes
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.get_mode().get_read() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.get_status().get_non_blocking() {
            return Ok((Pipe.Read(buffer).await?, *Underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            if let Ok(Size) = Pipe.Read(buffer).await {
                return Ok((Size, *Underlying_file));
            }
        }
    }

    pub async fn Read_line(
        &self,
        file: Local_file_identifier_type,
        buffer: &mut String,
    ) -> Result_type<(Size_type, Option<Unique_file_identifier_type>)> {
        let mut inner = self.0.write().await;

        let (Pipe, Flags, Underlying_file) = inner
            .open_pipes
            .get_mut(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.get_mode().get_read() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.get_status().get_non_blocking() {
            return Ok((Pipe.Read_line(buffer).await?, *Underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            if let Ok(Size) = Pipe.Read_line(buffer).await {
                return Ok((Size, *Underlying_file));
            }
        }
    }

    pub async fn Write(
        &self,
        file: Local_file_identifier_type,
        buffer: &[u8],
    ) -> Result_type<(Size_type, Option<Unique_file_identifier_type>)> {
        let inner = self.0.read().await;

        let (Pipe, Flags, Underlying_file) = inner
            .open_pipes
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.get_mode().get_write() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.get_status().get_non_blocking() {
            return Ok((Pipe.Write(buffer).await?, *Underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            if let Ok(Size) = Pipe.Write(buffer).await {
                return Ok((Size, *Underlying_file));
            }
        }
    }

    pub async fn get_mode(&self, File: Local_file_identifier_type) -> Result_type<Mode_type> {
        Ok(self
            .0
            .read()
            .await
            .open_pipes
            .get(&File)
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

        let result = fs.Create_unnamed_pipe(task_id, status, buffer_size).await;
        assert!(result.is_ok());

        let (read_file, write_file) = result.unwrap();
        assert!(fs.0.read().await.open_pipes.contains_key(&read_file));
        assert!(fs.0.read().await.open_pipes.contains_key(&write_file));
    }

    #[Test]
    async fn test_create_named_pipe() {
        let fs = File_system_type::new();
        let buffer_size = 1024;

        let result = fs.Create_named_pipe(buffer_size).await;
        assert!(result.is_ok());

        let inode = result.unwrap();
        assert!(fs.0.read().await.named_pipes.contains_key(&inode));
    }

    #[Test]
    async fn test_open_and_close_named_pipe() {
        let fs = File_system_type::new();
        let buffer_size = 1024;
        let task_id = Task_identifier_type::new(0);
        let flags = Flags_type::New(Mode_type::READ_WRITE, None, None);

        let inode = fs.Create_named_pipe(buffer_size).await.unwrap();
        let file_id = fs
            .Open(inode, task_id, flags, Unique_file_identifier_type::from(0))
            .await
            .unwrap();

        assert!(fs.0.read().await.open_pipes.contains_key(&file_id));

        fs.Close(file_id).await.unwrap();
        assert!(!fs.0.read().await.open_pipes.contains_key(&file_id));
    }

    #[Test]
    async fn test_duplicate_file_identifier() {
        let fs = File_system_type::new();
        let task_id = Task_identifier_type::new(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();

        let new_file = fs.Duplicate(read_file, None).await.unwrap();

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
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();
        let new_file = fs.Transfert(new_task_id, read_file, None).await.unwrap();

        assert!(fs.0.read().await.open_pipes.contains_key(&new_file));
        assert!(!fs.0.read().await.open_pipes.contains_key(&read_file));
    }

    #[Test]
    async fn test_delete_named_pipe() {
        let fs = File_system_type::new();
        let buffer_size = 1024;

        let inode = fs.Create_named_pipe(buffer_size).await.unwrap();
        fs.Remove(inode).await.unwrap();

        assert!(!fs.0.read().await.named_pipes.contains_key(&inode));
    }

    #[Test]
    async fn test_read_and_write_pipe() {
        let fs = File_system_type::new();
        let task_id = Task_identifier_type::new(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, write_file) = fs
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();

        // Write data to the pipe
        let data = b"hello";
        fs.Write(write_file, data).await.unwrap();

        // Read data from the pipe
        let mut buffer = [0; 5];
        fs.Read(read_file, &mut buffer).await.unwrap();
        assert_eq!(&buffer, b"hello");
    }

    #[Test]
    async fn test_get_mode() {
        let fs = File_system_type::new();
        let task_id = Task_identifier_type::new(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();
        let mode = fs.get_mode(read_file).await.unwrap();

        assert!(mode.get_read());
        assert!(!mode.get_write());
    }
}
