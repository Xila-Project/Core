use std::thread;

/// A wrapper around [std::thread::Thread].
pub struct Thread_wrapper_type(thread::JoinHandle<()>);

impl Thread_wrapper_type {
    /// Creates a new thread with a given name, stack size and function.
    pub fn New<F>(Name: &str, Stack_size: Option<usize>, Function: F) -> Result<Self, ()>
    where
        F: FnOnce() + Send + 'static,
    {
        let Thread_builder = thread::Builder::new().name(Name.to_string());

        let Thread_builder = match Stack_size {
            Some(Stack_size) => Thread_builder.stack_size(Stack_size),
            None => Thread_builder,
        };

        let Join_handle = match Thread_builder.spawn(Function) {
            Ok(Join_handle) => Join_handle,
            Err(_) => return Err(()),
        };

        Ok(Self(Join_handle))
    }

    /// Block the current thread until the thread terminates.
    pub fn Join(self) -> Result<(), ()> {
        match self.0.join() {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    /// Gets the name of the thread.
    pub fn Get_name(&self) -> Option<&str> {
        self.0.thread().name()
    }

    pub fn Sleep(Duration: std::time::Duration) {
        std::thread::sleep(Duration);
    }
}
