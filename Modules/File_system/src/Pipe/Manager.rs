use std::{collections::BTreeMap, sync::RwLock, time::Duration};

use Task::Task_identifier_type;

use crate::{
    Error_type, Flags_type, Get_new_file_identifier, Get_new_inode, Inode_type,
    Local_file_identifier_type, Mode_type, Result_type, Size_type, Status_type,
};

use super::Pipe_type;

struct Inner_type {
    pub Named_pipes: BTreeMap<Inode_type, Pipe_type>,
    pub Open_pipes: BTreeMap<Local_file_identifier_type, (Pipe_type, Flags_type)>,
}

pub struct File_system_type(RwLock<Inner_type>);

impl File_system_type {
    pub fn New() -> Self {
        Self(RwLock::new(Inner_type {
            Named_pipes: BTreeMap::new(),
            Open_pipes: BTreeMap::new(),
        }))
    }

    fn Create_unnamed_pipe(
        &self,
        Task: Task_identifier_type,
        Status: Status_type,
        Buffer_size: usize,
    ) -> Result_type<(Local_file_identifier_type, Local_file_identifier_type)> {
        let mut Inner = self.0.write()?;

        // Create the pipe
        let Pipe = Pipe_type::New(Buffer_size);

        // - Create the read file
        let Read_flags = Flags_type::New(Mode_type::Read_only, None, Some(Status));

        let Read_file = Get_new_file_identifier(Task, &Inner.Open_pipes)?;

        if Inner
            .Open_pipes
            .insert(Read_file, (Pipe.clone(), Read_flags))
            .is_some()
        {
            return Err(Error_type::Internal_error); // Should never happen
        }

        // - Create the write file
        let Write_flags = Flags_type::New(Mode_type::Write_only, None, Some(Status));

        let Write_file = Get_new_file_identifier(Task, &Inner.Open_pipes)?;

        if Inner
            .Open_pipes
            .insert(Write_file, (Pipe.clone(), Write_flags))
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
        &mut BTreeMap<Local_file_identifier_type, (Pipe_type, Flags_type)>,
    ) {
        (&mut Inner.Named_pipes, &mut Inner.Open_pipes)
    }

    pub fn Create_named_pipe(&self, Buffer_size: usize) -> Result_type<Inode_type> {
        let mut Inner = self.0.write()?;

        let Inode = Get_new_inode(&Inner.Named_pipes)?;

        let Pipe = Pipe_type::New(Buffer_size);

        if Inner.Named_pipes.insert(Inode, Pipe).is_some() {
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok(Inode)
    }

    fn Open(
        &self,
        Inode: Inode_type,
        Task: Task_identifier_type,
        Flags: Flags_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.0.write()?;

        let (Named_pipes, Open_pipes) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let Pipe = Named_pipes
            .get(&Inode)
            .ok_or(Error_type::Invalid_identifier)?;

        let Local_file_identifier = Get_new_file_identifier(Task, Open_pipes)?;

        Open_pipes.insert(Local_file_identifier, (Pipe.clone(), Flags));

        Ok(Local_file_identifier)
    }

    fn Close(&self, File: Local_file_identifier_type) -> Result_type<()> {
        self.0
            .write()?
            .Open_pipes
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(())
    }

    fn Close_all(&self, Task: Task_identifier_type) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        // Get all the keys of the open pipes that belong to the task
        let Keys = Inner
            .Open_pipes
            .keys()
            .filter(|Key| Key.Split().0 == Task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the pipes corresponding to the keys
        for Key in Keys {
            if let Some((Pipe, _)) = Inner.Open_pipes.remove(&Key) {
                drop(Pipe);
            }
        }

        Ok(())
    }

    fn Duplicate_file_identifier(
        &self,
        File: Local_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.0.write()?;

        let (Pipe, Flags) = Inner
            .Open_pipes
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .clone();

        let New_file = Get_new_file_identifier(File.Split().0, &Inner.Open_pipes)?;

        Inner
            .Open_pipes
            .insert(New_file, (Pipe.clone(), Flags.clone()));

        Ok(New_file)
    }

    fn Transfert_file_identifier(
        &self,
        New_task: Task_identifier_type,
        File: Local_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.0.write()?;

        let (Pipe, Flags) = Inner
            .Open_pipes
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        let New_file = Get_new_file_identifier(New_task, &Inner.Open_pipes)?;

        Inner.Open_pipes.insert(New_file, (Pipe, Flags));

        Ok(New_file)
    }

    fn Delete(&self, Inode: Inode_type) -> Result_type<()> {
        self.0
            .write()?
            .Named_pipes
            .remove(&Inode)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(())
    }

    fn Read(&self, File: Local_file_identifier_type, Buffer: &mut [u8]) -> Result_type<Size_type> {
        let Inner = self.0.read()?;

        let (Pipe, Flags) = Inner
            .Open_pipes
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.Get_mode().Get_read() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.Get_status().Get_non_blocking() {
            return Pipe.Read(Buffer);
        }

        loop {
            // Wait for the pipe to be ready
            if Pipe.Read(Buffer).is_ok() {
                return Ok(Buffer.len().into());
            }

            Task::Manager_type::Sleep(Duration::from_millis(1));
        }
    }

    fn Write(&self, File: Local_file_identifier_type, Buffer: &[u8]) -> Result_type<Size_type> {
        let Inner = self.0.read()?;

        let (Pipe, Flags) = Inner
            .Open_pipes
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.Get_mode().Get_write() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.Get_status().Get_non_blocking() {
            return Pipe.Write(Buffer);
        }

        loop {
            // Wait for the pipe to be ready
            if Pipe.Write(Buffer).is_ok() {
                return Ok(Buffer.len().into());
            }

            Task::Manager_type::Sleep(Duration::from_millis(1));
        }
    }

    fn Set_position(
        &self,
        _: Local_file_identifier_type,
        _: &crate::Position_type,
    ) -> Result_type<Size_type> {
        Err(Error_type::Unsupported_operation)
    }

    fn Flush(&self, _: Local_file_identifier_type) -> Result_type<()> {
        Ok(())
    }

    fn Get_mode(&self, File: Local_file_identifier_type) -> Result_type<Mode_type> {
        Ok(self
            .0
            .read()?
            .Open_pipes
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .1
            .Get_mode()
            .clone())
    }
}

#[cfg(test)]
mod Tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn Test_create_unnamed_pipe() {
        let fs = File_system_type::New();
        let task_id = Task_identifier_type::New(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let result = fs.Create_unnamed_pipe(task_id, status, buffer_size);
        assert!(result.is_ok());

        let (read_file, write_file) = result.unwrap();
        assert!(fs.0.read().unwrap().Open_pipes.contains_key(&read_file));
        assert!(fs.0.read().unwrap().Open_pipes.contains_key(&write_file));
    }

    #[test]
    fn Test_create_named_pipe() {
        let fs = File_system_type::New();
        let buffer_size = 1024;

        let result = fs.Create_named_pipe(buffer_size);
        assert!(result.is_ok());

        let inode = result.unwrap();
        assert!(fs.0.read().unwrap().Named_pipes.contains_key(&inode));
    }

    #[test]
    fn Test_open_and_close_named_pipe() {
        let fs = File_system_type::New();
        let buffer_size = 1024;
        let task_id = Task_identifier_type::New(0);
        let flags = Flags_type::New(Mode_type::Read_write, None, None);

        let inode = fs.Create_named_pipe(buffer_size).unwrap();
        let file_id = fs.Open(inode, task_id, flags).unwrap();

        assert!(fs.0.read().unwrap().Open_pipes.contains_key(&file_id));

        fs.Close(file_id).unwrap();
        assert!(!fs.0.read().unwrap().Open_pipes.contains_key(&file_id));
    }

    #[test]
    fn Test_duplicate_file_identifier() {
        let fs = File_system_type::New();
        let task_id = Task_identifier_type::New(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .unwrap();
        let new_file = fs.Duplicate_file_identifier(read_file).unwrap();

        assert!(fs.0.read().unwrap().Open_pipes.contains_key(&new_file));
    }

    #[test]
    fn Test_transfert_file_identifier() {
        let fs = File_system_type::New();
        let task_id = Task_identifier_type::New(0);
        let new_task_id = Task_identifier_type::New(1);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .unwrap();
        let new_file = fs
            .Transfert_file_identifier(new_task_id, read_file)
            .unwrap();

        assert!(fs.0.read().unwrap().Open_pipes.contains_key(&new_file));
        assert!(!fs.0.read().unwrap().Open_pipes.contains_key(&read_file));
    }

    #[test]
    fn Test_delete_named_pipe() {
        let fs = File_system_type::New();
        let buffer_size = 1024;

        let inode = fs.Create_named_pipe(buffer_size).unwrap();
        fs.Delete(inode).unwrap();

        assert!(!fs.0.read().unwrap().Named_pipes.contains_key(&inode));
    }

    #[test]
    fn Test_read_and_write_pipe() {
        let fs = Arc::new(File_system_type::New());
        let task_id = Task_identifier_type::New(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, write_file) = fs
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .unwrap();

        let fs_clone = Arc::clone(&fs);
        let writer = thread::spawn(move || {
            let data = b"hello";
            fs_clone.Write(write_file, data).unwrap();
        });

        let fs_clone = Arc::clone(&fs);
        let reader = thread::spawn(move || {
            let mut buffer = [0; 5];
            fs_clone.Read(read_file, &mut buffer).unwrap();
            assert_eq!(&buffer, b"hello");
        });

        writer.join().unwrap();
        reader.join().unwrap();
    }

    #[test]
    fn Test_set_position_unsupported() {
        let fs = File_system_type::New();
        let task_id = Task_identifier_type::New(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .unwrap();

        let result = fs.Set_position(read_file, &crate::Position_type::Start(0));
        assert_eq!(result, Err(Error_type::Unsupported_operation));
    }

    #[test]
    fn Test_flush() {
        let fs = File_system_type::New();
        let task_id = Task_identifier_type::New(0);
        let status = Status_type::default();

        let buffer_size = 1024;

        let (read_file, _) = fs
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .unwrap();

        let result = fs.Flush(read_file);
        assert!(result.is_ok());
    }

    #[test]
    fn Test_get_mode() {
        let fs = File_system_type::New();
        let task_id = Task_identifier_type::New(0);
        let status = Status_type::default();
        let buffer_size = 1024;

        let (read_file, _) = fs
            .Create_unnamed_pipe(task_id, status, buffer_size)
            .unwrap();
        let mode = fs.Get_mode(read_file).unwrap();

        assert!(mode.Get_read());
        assert!(!mode.Get_write());
    }
}
