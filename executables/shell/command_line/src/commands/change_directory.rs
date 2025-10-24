use alloc::{borrow::ToOwned, format};
use xila::{file_system::Path, virtual_file_system};

use crate::Shell;

impl Shell {
    pub async fn change_directory(&mut self, arguments: &[&str]) {
        if arguments.len() != 1 {
            self.standard
                .print_error_line("Invalid number of arguments")
                .await;
        }

        let current_directory = Path::from_str(arguments[0]).to_owned();

        let current_directory = if current_directory.is_absolute() {
            current_directory
        } else {
            match self.current_directory.clone().join(&current_directory) {
                Some(path) => path.canonicalize(),
                None => {
                    self.standard.print_error_line("Failed to join paths").await;
                    return;
                }
            }
        };

        if let Err(error) = virtual_file_system::get_instance()
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
