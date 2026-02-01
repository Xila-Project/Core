use xila::task::TaskIdentifier;

#[derive(Debug, Clone)]
pub struct CustomData {
    task_identifier: TaskIdentifier,
}

impl CustomData {
    pub const fn new(task: TaskIdentifier) -> Self {
        Self {
            task_identifier: task,
        }
    }

    pub const fn get_task_identifier(&self) -> TaskIdentifier {
        self.task_identifier
    }
}
