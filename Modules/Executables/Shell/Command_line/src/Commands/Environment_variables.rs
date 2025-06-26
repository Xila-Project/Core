use alloc::format;

use crate::Shell_type;

impl Shell_type {
    pub async fn Set_environment_variable(&mut self, Arguments: &[&str]) {
        if Arguments.len() != 1 {
            self.Standard
                .Print_error_line("Invalid number of arguments")
                .await;
            return;
        }

        let (Name, Value) = match Arguments[0].split_once('=') {
            Some((Name, Value)) => (Name, Value),
            None => {
                self.Standard.Print_error_line("Invalid argument").await;
                return;
            }
        };

        if let Err(Error) = Task::Get_instance()
            .Set_environment_variable(self.Standard.Get_task(), Name, Value)
            .await
        {
            self.Standard
                .Print_error_line(&format!("Failed to set environment variable: {}", Error))
                .await;
        }
    }

    pub async fn Remove_environment_variable(&mut self, Arguments: &[&str]) {
        if Arguments.len() != 1 {
            self.Standard
                .Print_error_line("Invalid number of arguments")
                .await;
            return;
        }

        let Name = Arguments[0];

        if let Err(Error) = Task::Get_instance()
            .Remove_environment_variable(self.Standard.Get_task(), Name)
            .await
        {
            self.Standard
                .Print_error_line(&format!("Failed to unset environment variable: {}", Error))
                .await;
        }
    }
}
