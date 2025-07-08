use task::Task_identifier_type;

#[derive(Debug, Clone)]
pub struct Custom_data_type {
    task_identifier: Task_identifier_type,
}

impl Custom_data_type {
    pub const fn new(task: Task_identifier_type) -> Self {
        Self {
            task_identifier: task,
        }
    }

    pub const fn get_task_identifier(&self) -> Task_identifier_type {
        self.task_identifier
    }
}
