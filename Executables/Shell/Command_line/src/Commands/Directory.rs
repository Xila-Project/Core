use alloc::{borrow::ToOwned, format};
use File_system::Path_type;

use crate::Shell_type;

impl Shell_type {
    pub async fn Create_directory(&mut self, Arguments: &[&str]) {
        if Arguments.len() != 1 {
            self.Standard
                .Print_error_line("Invalid number of arguments")
                .await;
            return;
        }

        let Path = Path_type::From_str(Arguments[0]);

        if !Path.Is_valid() {
            self.Standard.Print_error_line("Invalid path").await;
            return;
        }

        let Path = if Path.Is_absolute() {
            Path.to_owned()
        } else {
            match self.Current_directory.clone().Join(Path) {
                Some(Path) => Path.Canonicalize(),
                None => {
                    self.Standard.Print_error_line("Failed to join paths").await;
                    return;
                }
            }
        };

        if let Err(Error) = Virtual_file_system::Get_instance()
            .Create_directory(&Path, self.Standard.Get_task())
            .await
        {
            self.Standard
                .Print_error_line(&format!("Failed to create directory: {Error}"))
                .await;
        }
    }

    pub async fn Remove(&mut self, Arguments: &[&str]) {
        if Arguments.len() != 1 {
            self.Standard
                .Print_error_line("Invalid number of arguments")
                .await;
            return;
        }

        let Path = Path_type::From_str(Arguments[0]);

        if !Path.Is_valid() {
            self.Standard.Print_error_line("Invalid path").await;
            return;
        }

        let Path = if Path.Is_absolute() {
            Path.to_owned()
        } else {
            match self.Current_directory.clone().Join(Path) {
                Some(Path) => Path.Canonicalize(),
                None => {
                    self.Standard.Print_error_line("Failed to join paths").await;
                    return;
                }
            }
        };

        if let Err(Error) = Virtual_file_system::Get_instance().Remove(&Path).await {
            self.Standard
                .Print_error_line(&format!("Failed to remove directory: {Error}"))
                .await;
        }
    }
}
