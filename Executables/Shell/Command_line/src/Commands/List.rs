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

        let Directory = match Virtual_file_system::Get_instance()
            .Open_directory(&path, self.standard.Get_task())
            .await
        {
            Ok(Directory) => Directory,
            Err(error) => {
                self.standard
                    .Print_error_line(&format!("Failed to open directory: {error:?}"))
                    .await;

                return;
            }
        };

        while let Ok(Some(Entry)) = Virtual_file_system::Get_instance()
            .Read_directory(Directory, self.standard.Get_task())
            .await
        {
            self.standard.Print(Entry.Get_name()).await;
            self.standard.Print("\n").await;
        }
    }
}
