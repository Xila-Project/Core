use alloc::format;
use file_system::{Mode_type, Path_type};

use crate::Shell_type;

impl Shell_type {
    async fn read_file_and_write(&mut self, path: &Path_type) {
        let file = match virtual_file_system::get_instance()
            .open(&path, Mode_type::READ_ONLY.into(), self.standard.get_task())
            .await
        {
            Ok(file) => file,
            Err(error) => {
                self.standard
                    .print_error_line(&format!("Failed to open file: {error:?}"))
                    .await;
                return;
            }
        };

        let mut buffer = [0_u8; 128];
        while let Ok(size) = virtual_file_system::get_instance()
            .read(file, &mut buffer, self.standard.get_task())
            .await
        {
            if size == 0 {
                break;
            }

            let size: usize = size.into();

            self.standard.write(&buffer[..size]).await;
        }
    }

    pub async fn concatenate(&mut self, arguments: &[&str]) {
        for path in arguments {
            let path = Path_type::From_str(path);

            if path.is_absolute() {
                self.read_file_and_write(path).await;
            } else {
                match self.current_directory.clone().Join(path) {
                    Some(path) => self.read_file_and_write(&path).await,
                    None => self.standard.print_error_line("Invalid command").await,
                }
            }
        }
    }
}
