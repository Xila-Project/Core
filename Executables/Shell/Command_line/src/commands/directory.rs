use alloc::{borrow::ToOwned, format};
use file_system::Path_type;

use crate::Shell_type;

impl Shell_type {
    pub async fn create_directory(&mut self, arguments: &[&str]) {
        if arguments.len() != 1 {
            self.standard
                .print_error_line("Invalid number of arguments")
                .await;
            return;
        }

        let path = Path_type::from_str(arguments[0]);

        if !path.is_valid() {
            self.standard.print_error_line("Invalid path").await;
            return;
        }

        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            match self.current_directory.clone().join(path) {
                Some(path) => path.canonicalize(),
                None => {
                    self.standard.print_error_line("Failed to join paths").await;
                    return;
                }
            }
        };

        if let Err(error) = virtual_file_system::get_instance()
            .create_directory(&path, self.standard.get_task())
            .await
        {
            self.standard
                .print_error_line(&format!("Failed to create directory: {error}"))
                .await;
        }
    }

    pub async fn remove(&mut self, arguments: &[&str]) {
        if arguments.len() != 1 {
            self.standard
                .print_error_line("Invalid number of arguments")
                .await;
            return;
        }

        let path = Path_type::from_str(arguments[0]);

        if !path.is_valid() {
            self.standard.print_error_line("Invalid path").await;
            return;
        }

        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            match self.current_directory.clone().join(path) {
                Some(path) => path.canonicalize(),
                None => {
                    self.standard.print_error_line("Failed to join paths").await;
                    return;
                }
            }
        };

        if let Err(error) = virtual_file_system::get_instance().remove(&path).await {
            self.standard
                .print_error_line(&format!("Failed to remove directory: {error}"))
                .await;
        }
    }
}
