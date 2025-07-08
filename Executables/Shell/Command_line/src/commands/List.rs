use alloc::format;
use File_system::Path_type;

use crate::Shell_type;

impl Shell_type {
    pub async fn list(&mut self, Arguments: &[&str]) {
        let path = if Arguments.is_empty() {
            self.current_directory.as_ref()
        } else {
            Path_type::From_str(Arguments[0])
        };

        let Directory = match Virtual_file_system::get_instance()
            .open_directory(&path, self.standard.get_task())
            .await
        {
            Ok(Directory) => Directory,
            Err(error) => {
                self.standard
                    .print_error_line(&format!("Failed to open directory: {error:?}"))
                    .await;

                return;
            }
        };

        while let Ok(Some(Entry)) = Virtual_file_system::get_instance()
            .read_directory(Directory, self.standard.get_task())
            .await
        {
            self.standard.print(Entry.get_name()).await;
            self.standard.print("\n").await;
        }
    }
}
