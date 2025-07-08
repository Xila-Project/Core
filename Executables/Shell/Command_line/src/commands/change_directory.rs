use alloc::{borrow::ToOwned, format};
use file_system::Path_type;

use crate::Shell_type;

impl Shell_type {
    pub async fn change_directory(&mut self, arguments: &[&str]) {
        if arguments.len() != 1 {
            self.standard
                .print_error_line("Invalid number of arguments")
                .await;
        }

        let current_directory = Path_type::From_str(arguments[0]).to_owned();

        let current_directory = if current_directory.is_absolute() {
            current_directory
        } else {
            match self.current_directory.clone().Join(&current_directory) {
                Some(path) => path.Canonicalize(),
                None => {
                    self.standard.print_error_line("Failed to join paths").await;
                    return;
                }
            }
        };

        if let Err(error) = Virtual_file_system::get_instance()
            .open_directory(&current_directory, self.standard.get_task())
            .await
        {
            self.standard
                .print_error_line(&format!("Failed to change directory: {error}"))
                .await;
            return;
        }

        self.current_directory = current_directory.to_owned();
    }
}
