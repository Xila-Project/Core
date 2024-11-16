use Task::Task_identifier_type;

#[derive(Debug, Clone)]
pub struct Custom_data_type {
    Task_identifier: Task_identifier_type,
}

impl Custom_data_type {
    pub const fn New(Task: Task_identifier_type) -> Self {
        Self {
            Task_identifier: Task,
        }
    }

    pub const fn Get_task_identifier(&self) -> Task_identifier_type {
        self.Task_identifier
    }
}
