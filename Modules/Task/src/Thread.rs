use super::*;
use std::{
    any::Any,
    mem::transmute,
    thread::{self},
};

pub struct Join_handle_type<T>(thread::JoinHandle<T>);

impl<T> Join_handle_type<T> {
    pub fn Join(self) -> std::result::Result<T, Box<dyn Any + Send>> {
        self.0.join()
    }

    pub(crate) fn Get_thread_wrapper(&self) -> Thread_wrapper_type {
        Thread_wrapper_type(self.0.thread().clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Thread_identifier_type(usize);

impl From<usize> for Thread_identifier_type {
    fn from(Identifier: usize) -> Self {
        Thread_identifier_type(Identifier)
    }
}

impl From<Thread_identifier_type> for usize {
    fn from(Identifier: Thread_identifier_type) -> Self {
        Identifier.0
    }
}

impl From<thread::ThreadId> for Thread_identifier_type {
    fn from(Identifier: thread::ThreadId) -> Self {
        let Identifier: u64 = unsafe { transmute(Identifier) };

        Thread_identifier_type(Identifier as usize)
    }
}

/// A wrapper around [std::thread::Thread].
pub struct Thread_wrapper_type(thread::Thread);

impl Thread_wrapper_type {
    /// Creates a new thread with a given name, stack size and function.
    pub fn Spawn<F, T>(
        Name: &str,
        Stack_size: Option<usize>,
        Function: F,
    ) -> Result_type<Join_handle_type<T>>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let Thread_builder = thread::Builder::new().name(Name.to_string());

        let Thread_builder = match Stack_size {
            Some(Stack_size) => Thread_builder.stack_size(Stack_size),
            None => Thread_builder,
        };

        let Join_handle = Thread_builder
            .spawn(Function)
            .map_err(|_| Error_type::Failed_to_spawn_thread)?;

        Ok(Join_handle_type(Join_handle))
    }

    /// Gets the name of the thread.
    pub fn Get_name(&self) -> Option<&str> {
        self.0.name()
    }

    pub fn Sleep(Duration: std::time::Duration) {
        std::thread::sleep(Duration);
    }

    pub fn Get_identifier(&self) -> Thread_identifier_type {
        Thread_identifier_type::from(self.0.id())
    }

    pub fn Get_current() -> Thread_wrapper_type {
        Thread_wrapper_type(thread::current())
    }
}
