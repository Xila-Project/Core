pub(crate) use alloc::format;

use crate::Shell_type;

impl Shell_type {
    pub async fn set_environment_variable(&mut self, arguments: &[&str]) {
        if arguments.len() != 1 {
            self.standard
                .print_error_line("Invalid number of arguments")
                .await;
            return;
        }

        let (name, value) = match arguments[0].split_once('=') {
            Some((name, value)) => (name, value),
            None => {
                self.standard.print_error_line("Invalid argument").await;
                return;
            }
        };

        if let Err(error) = task::get_instance()
            .set_environment_variable(self.standard.get_task(), name, value)
            .await
        {
            self.standard
                .print_error_line(&format!("Failed to set environment variable: {error}"))
                .await;
        }
    }

    pub async fn remove_environment_variable(&mut self, arguments: &[&str]) {
        if arguments.len() != 1 {
            self.standard
                .print_error_line("Invalid number of arguments")
                .await;
            return;
        }

        let name = arguments[0];

        if let Err(error) = task::get_instance()
            .remove_environment_variable(self.standard.get_task(), name)
            .await
        {
            self.standard
                .print_error_line(&format!("Failed to unset environment variable: {error}"))
                .await;
        }
    }
}
