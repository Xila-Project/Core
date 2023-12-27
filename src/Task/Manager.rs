// - Libraries
use super::*;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

struct Task_internal_type {
    Thread: Thread_wrapper_type,
    Children: Vec<Task_identifier_type>,
}

pub struct Manager_type {
    Tasks: Arc<RwLock<HashMap<Task_identifier_type, Task_internal_type>>>,
}

impl Manager_type {
    const Root_task_identifier: Task_identifier_type = 0;

    pub fn New<F>(Main_task_function: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Manager_type {
            Tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn Get_new_task_identifier(&self) -> Task_identifier_type {
        if self.Tasks.read().unwrap().len() == 0 {
            return Self::Root_task_identifier;
        }

        for Process_identifier in 0..std::usize::MAX - 1 {
            if !self.Tasks.read().unwrap().contains_key(&Process_identifier) {
                return Process_identifier;
            }
        }
        panic!("No more process identifier available."); // Crazy shit
    }

    pub fn Get_task_name(&self, Process_identifier: Task_identifier_type) -> Result<String, ()> {
        match self.Tasks.read().unwrap().get(&Process_identifier) {
            Some(Task) => match Task.Thread.Get_name() {
                Some(Name) => Ok(Name.to_string()),
                None => Err(()),
            },
            None => Err(()),
        }
    }

    pub fn Get_process_child_processes(
        &self,
        Process_identifier: Task_identifier_type,
    ) -> Result<Vec<Task_identifier_type>, ()> {
        match self.Tasks.read().unwrap().get(&Process_identifier) {
            Some(Process) => Ok(Process.Children.clone()),
            None => Err(()),
        }
    }

    pub fn Terminate_task(&self, Task_identifier: Task_identifier_type) {
        todo!()
    }

    pub fn New_task<F>(
        &self,
        Parent_task_identifier: Task_identifier_type,
        Name: &str,
        Stack_size: Option<usize>,
        Function: F,
    ) -> Result<Task_identifier_type, ()>
    where
        F: FnOnce() + Send + 'static,
    {
        let mut Tasks = self.Tasks.write().unwrap(); // Acquire lock

        let mut Parent_task = match Tasks.get_mut(&Parent_task_identifier) {
            Some(Parent_task) => Parent_task,
            None => return Err(()),
        };

        let Thread_wrapper = match Thread_wrapper_type::New(Name, Stack_size, Function) {
            Ok(Thread_wrapper) => Thread_wrapper,
            Err(()) => return Err(()),
        };

        let Child_task_identifier = self.Get_new_task_identifier();

        let mut Tasks = self.Tasks.write().unwrap(); // Acquire lock again

        Tasks.insert(
            Child_task_identifier,
            Task_internal_type {
                Thread: Thread_wrapper,
                Children: Vec::new(),
            },
        );

        Parent_task.Children.push(Child_task_identifier);

        Ok(Child_task_identifier)
    }
}
