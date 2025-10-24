use alloc::format;
use xila::{file_system::Path, virtual_file_system};

use crate::Shell;

impl Shell {
    pub async fn list(&mut self, arguments: &[&str]) {
        let path = if arguments.is_empty() {
            self.current_directory.as_ref()
        } else {
            Path::from_str(arguments[0])
        };

        let directory = match virtual_file_system::get_instance()
            .open_directory(&path, self.standard.get_task())
            .await
        {
            Ok(directory) => directory,
            Err(error) => {
                self.standard
                    .print_error_line(&format!("Failed to open directory: {error:?}"))
                    .await;

                return;
            }
        };

        while let Ok(Some(entry)) = virtual_file_system::get_instance()
            .read_directory(directory, self.standard.get_task())
            .await
        {
            self.standard.print(entry.get_name()).await;
            self.standard.print("\n").await;
        }
    }
}
