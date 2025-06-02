extern crate alloc;

use alloc::collections::BTreeMap;

use Synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use Task::Task_identifier_type;

use File_system::{
    Error_type, File_identifier_type, Flags_type, Get_new_file_identifier, Get_new_inode,
    Inode_type, Local_file_identifier_type, Mode_type, Result_type, Size_type, Status_type,
    Unique_file_identifier_type,
};

use super::Pipe_type;

type Open_pipes_inner_type = (Pipe_type, Flags_type, Option<Unique_file_identifier_type>);

struct Inner_type {
    pub Named_pipes: BTreeMap<Inode_type, Pipe_type>,
    pub Open_pipes: BTreeMap<Local_file_identifier_type, Open_pipes_inner_type>,
}

pub struct File_system_type(RwLock<CriticalSectionRawMutex, Inner_type>);

impl File_system_type {
    pub async fn New() -> Self {
        Self(RwLock::new(Inner_type {
            Named_pipes: BTreeMap::new(),
            Open_pipes: BTreeMap::new(),
        }))
    }

    pub async fn Get_underlying_file(
        &self,
        File: Local_file_identifier_type,
    ) -> Result_type<Option<Unique_file_identifier_type>> {
        Ok(self
            .0
            .read()
            .await
            .Open_pipes
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .2)
    }

    pub async fn Create_unnamed_pipe(
        &self,
        Task: Task_identifier_type,
        Status: Status_type,
        Buffer_size: usize,
    ) -> Result_type<(Local_file_identifier_type, Local_file_identifier_type)> {
        let mut Inner = self.0.write().await;

        // Create the pipe
        let Pipe = Pipe_type::New(Buffer_size);

        // - Create the read file
        let Read_flags = Flags_type::New(Mode_type::Read_only, None, Some(Status));

        let Read_file = Get_new_file_identifier(Task, None, None, &Inner.Open_pipes)?;

        if Inner
            .Open_pipes
            .insert(Read_file, (Pipe.clone(), Read_flags, None))
            .is_some()
        {
            return Err(Error_type::Internal_error); // Should never happen
        }

        // - Create the write file
        let Write_flags = Flags_type::New(Mode_type::Write_only, None, Some(Status));

        let Write_file = Get_new_file_identifier(Task, None, None, &Inner.Open_pipes)?;

        if Inner
            .Open_pipes
            .insert(Write_file, (Pipe.clone(), Write_flags, None))
            .is_some()
        {
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok((Read_file, Write_file))
    }

    fn Borrow_mutable_inner_2_splited(
        Inner: &mut Inner_type,
    ) -> (
        &mut BTreeMap<Inode_type, Pipe_type>,
        &mut BTreeMap<Local_file_identifier_type, Open_pipes_inner_type>,
    ) {
        (&mut Inner.Named_pipes, &mut Inner.Open_pipes)
    }

    pub async fn Create_named_pipe(&self, Buffer_size: usize) -> Result_type<Inode_type> {
        let mut Inner = self.0.write().await;

        let Inode = Get_new_inode(&Inner.Named_pipes)?;

        let Pipe = Pipe_type::New(Buffer_size);

        if Inner.Named_pipes.insert(Inode, Pipe).is_some() {
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok(Inode)
    }

    pub async fn Open(
        &self,
        Inode: Inode_type,
        Task: Task_identifier_type,
        Flags: Flags_type,
        Underlying_file: Unique_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.0.write().await;

        let (Named_pipes, Open_pipes) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let Pipe = Named_pipes
            .get(&Inode)
            .ok_or(Error_type::Invalid_identifier)?;

        let Local_file_identifier = Get_new_file_identifier(Task, None, None, Open_pipes)?;

        Open_pipes.insert(
            Local_file_identifier,
            (Pipe.clone(), Flags, Some(Underlying_file)),
        );

        Ok(Local_file_identifier)
    }

    pub async fn Close(
        &self,
        File: Local_file_identifier_type,
    ) -> Result_type<Option<Unique_file_identifier_type>> {
        let (_, _, Underlying_file) = self
            .0
            .write()
            .await
            .Open_pipes
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(Underlying_file)
    }

    pub async fn Close_all(&self, Task: Task_identifier_type) -> Result_type<()> {
        let mut Inner = self.0.write().await;

        // Get all the keys of the open pipes that belong to the task
        let Keys = Inner
            .Open_pipes
            .keys()
            .filter(|Key| Key.Split().0 == Task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the pipes corresponding to the keys
        for Key in Keys {
            if let Some((Pipe, _, _)) = Inner.Open_pipes.remove(&Key) {
                drop(Pipe);
            }
        }

        Ok(())
    }

    pub async fn Duplicate(
        &self,
        File: Local_file_identifier_type,
        Underlying_file: Option<Unique_file_identifier_type>,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.0.write().await;

        let (Pipe, Flags, _) = Inner
            .Open_pipes
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .clone();

        let New_file = Get_new_file_identifier(File.Split().0, None, None, &Inner.Open_pipes)?;

        Inner
            .Open_pipes
            .insert(New_file, (Pipe.clone(), Flags, Underlying_file));

        Ok(New_file)
    }

    pub async fn Transfert(
        &self,
        New_task: Task_identifier_type,
        File: Local_file_identifier_type,
        New_file: Option<File_identifier_type>,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.0.write().await;

        let (Pipe, Flags, Underlying_file) = Inner
            .Open_pipes
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        let New_file = if let Some(File) = New_file {
            let File = Local_file_identifier_type::New(New_task, File);

            if Inner.Open_pipes.contains_key(&File) {
                return Err(Error_type::Invalid_identifier);
            }

            File
        } else {
            Get_new_file_identifier(New_task, None, None, &Inner.Open_pipes)?
        };

        if Inner
            .Open_pipes
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
            .Named_pipes
            .remove(&Inode)
            .ok_or(Error_type::Invalid_inode)?;

        Ok(())
    }

    pub async fn Read(
        &self,
        File: Local_file_identifier_type,
        Buffer: &mut [u8],
    ) -> Result_type<(Size_type, Option<Unique_file_identifier_type>)> {
        let Inner = self.0.read().await;

        let (Pipe, Flags, Underlying_file) = Inner
            .Open_pipes
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.Get_mode().Get_read() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.Get_status().Get_non_blocking() {
            return Ok((Pipe.Read(Buffer).await?, *Underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            if let Ok(Size) = Pipe.Read(Buffer).await {
                return Ok((Size, *Underlying_file));
            }
        }
    }

    pub async fn Read_line(
        &self,
        File: Local_file_identifier_type,
        Buffer: &mut String,
    ) -> Result_type<(Size_type, Option<Unique_file_identifier_type>)> {
        let mut Inner = self.0.write().await;

        let (Pipe, Flags, Underlying_file) = Inner
            .Open_pipes
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.Get_mode().Get_read() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.Get_status().Get_non_blocking() {
            return Ok((Pipe.Read_line(Buffer).await?, *Underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            if let Ok(Size) = Pipe.Read_line(Buffer).await {
                return Ok((Size, *Underlying_file));
            }
        }
    }

    pub async fn Write(
        &self,
        File: Local_file_identifier_type,
        Buffer: &[u8],
    ) -> Result_type<(Size_type, Option<Unique_file_identifier_type>)> {
        let Inner = self.0.read().await;

        let (Pipe, Flags, Underlying_file) = Inner
            .Open_pipes
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.Get_mode().Get_write() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.Get_status().Get_non_blocking() {
            return Ok((Pipe.Write(Buffer).await?, *Underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            if let Ok(Size) = Pipe.Write(Buffer).await {
                return Ok((Size, *Underlying_file));
            }
        }
    }

    pub async fn Get_mode(&self, File: Local_file_identifier_type) -> Result_type<Mode_type> {
        Ok(self
            .0
            .read()
            .await
            .Open_pipes
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .1
            .Get_mode())
    }
}

#[cfg(test)]
mod Tests {
    use Task::Test;

    use super::*;

    extern crate alloc;

    #[Test]
    async fn Test_create_unnamed_pipe() {
        let fs = File_system_type::New().await;
        let task_id = Task_identifier_type::New(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let result = fs.Create_unnamed_pipe(task_id, status, buffer_size).await;
        assert!(result.is_ok());

        let (read_file, write_file) = result.unwrap();
        assert!(fs.0.read().await.Open_pipes.contains_key(&read_file));
        assert!(fs.0.read().await.Open_pipes.contains_key(&write_file));
    }

    #[Test]
    async fn Test_create_named_pipe() {
        let fs = File_system_type::New().await;
        let buffer_size = 1024;

        let result = fs.Create_named_pipe(buffer_size).await;
        assert!(result.is_ok());

        let inode = result.unwrap();
        assert!(fs.0.read().await.Named_pipes.contains_key(&inode));
    }

    #[Test]
    async fn Test_open_and_close_named_pipe() {
        let fs = File_system_type::New().await;
        let buffer_size = 1024;
        let task_id = Task_identifier_type::New(0);
        let flags = Flags_type::New(Mode_type::Read_write, None, None);

        let inode = fs.Create_named_pipe(buffer_size).await.unwrap();
        let file_id = fs
            .Open(inode, task_id, flags, Unique_file_identifier_type::from(0))
            .await
            .unwrap();

        assert!(fs.0.read().await.Open_pipes.contains_key(&file_id));

        fs.Close(file_id).await.unwrap();
        assert!(!fs.0.read().await.Open_pipes.contains_key(&file_id));
    }

    #[Test]
    async fn Test_duplicate_file_identifier() {
        let fs = File_system_type::New().await;
        let task_id = Task_identifier_type::New(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();

        let new_file = fs.Duplicate(read_file, None).await.unwrap();

        assert!(fs.0.read().await.Open_pipes.contains_key(&new_file));
    }

    #[Test]
    async fn Test_transfert_file_identifier() {
        let fs = File_system_type::New().await;
        let task_id = Task_identifier_type::New(0);
        let new_task_id = Task_identifier_type::New(1);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();
        let new_file = fs.Transfert(new_task_id, read_file, None).await.unwrap();

        assert!(fs.0.read().await.Open_pipes.contains_key(&new_file));
        assert!(!fs.0.read().await.Open_pipes.contains_key(&read_file));
    }

    #[Test]
    async fn Test_delete_named_pipe() {
        let fs = File_system_type::New().await;
        let buffer_size = 1024;

        let inode = fs.Create_named_pipe(buffer_size).await.unwrap();
        fs.Remove(inode).await.unwrap();

        assert!(!fs.0.read().await.Named_pipes.contains_key(&inode));
    }

    #[Test]
    async fn Test_read_and_write_pipe() {
        let fs = File_system_type::New().await;
        let task_id = Task_identifier_type::New(0);
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
    async fn Test_get_mode() {
        let fs = File_system_type::New().await;
        let task_id = Task_identifier_type::New(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .await
            .unwrap();
        let mode = fs.Get_mode(read_file).await.unwrap();

        assert!(mode.Get_read());
        assert!(!mode.Get_write());
    }
}
