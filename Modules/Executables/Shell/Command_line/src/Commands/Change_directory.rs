use alloc::{borrow::ToOwned, format};
use File_system::Path_type;

use crate::Shell_type;

impl Shell_type {
    pub async fn Change_directory(&mut self, Arguments: &[&str]) {
        if Arguments.len() != 1 {
            self.Standard
                .Print_error_line("Invalid number of arguments")
                .await;
        }

        let Current_directory = Path_type::From_str(Arguments[0]).to_owned();

        let Current_directory = if Current_directory.Is_absolute() {
            Current_directory
        } else {
            match self.Current_directory.clone().Join(&Current_directory) {
                Some(Path) => Path.Canonicalize(),
                None => {
                    self.Standard.Print_error_line("Failed to join paths").await;
                    return;
                }
            }
        };

        if let Err(Error) = Virtual_file_system::Get_instance()
            .Open_directory(&Current_directory, self.Standard.Get_task())
            .await
        {
            self.Standard
                .Print_error_line(&format!("Failed to change directory: {Error}"))
                .await;
            return;
        }

        self.Current_directory = Current_directory.to_owned();
    }
}
