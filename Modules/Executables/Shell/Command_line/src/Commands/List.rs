use alloc::format;
use File_system::Path_type;

use crate::Shell_type;

impl Shell_type {
    pub async fn List(&mut self, Arguments: &[&str]) {
        let Path = if Arguments.is_empty() {
            self.Current_directory.as_ref()
        } else {
            Path_type::From_str(Arguments[0])
        };

        let Directory = match Virtual_file_system::Get_instance()
            .Open_directory(&Path, self.Standard.Get_task())
            .await
        {
            Ok(Directory) => Directory,
            Err(Error) => {
                self.Standard
                    .Print_error_line(&format!("Failed to open directory: {Error:?}"))
                    .await;

                return;
            }
        };

        while let Ok(Some(Entry)) = Virtual_file_system::Get_instance()
            .Read_directory(Directory, self.Standard.Get_task())
            .await
        {
            self.Standard.Print(Entry.Get_name()).await;
            self.Standard.Print("\n").await;
        }
    }
}
