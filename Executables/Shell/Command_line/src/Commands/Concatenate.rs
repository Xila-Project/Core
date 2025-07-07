use alloc::format;
use File_system::{Mode_type, Path_type};

use crate::Shell_type;

impl Shell_type {
    async fn read_file_and_write(&mut self, Path: &Path_type) {
        let file = match Virtual_file_system::Get_instance()
            .Open(&Path, Mode_type::READ_ONLY.into(), self.standard.Get_task())
            .await
        {
            Ok(File) => File,
            Err(error) => {
                self.standard
                    .Print_error_line(&format!("Failed to open file: {error:?}"))
                    .await;
                return;
            }
        };

        let mut Buffer = [0_u8; 128];
        while let Ok(size) = Virtual_file_system::Get_instance()
            .Read(file, &mut Buffer, self.standard.Get_task())
            .await
        {
            if size == 0 {
                break;
            }

            let Size: usize = size.into();

            self.standard.Write(&Buffer[..Size]).await;
        }
    }

    pub async fn Concatenate(&mut self, Arguments: &[&str]) {
        for path in Arguments {
            let path = Path_type::From_str(path);

            if path.Is_absolute() {
                self.read_file_and_write(path).await;
            } else {
                match self.current_directory.clone().Join(path) {
                    Some(path) => self.read_file_and_write(&path).await,
                    None => self.standard.Print_error_line("Invalid command").await,
                }
            }
        }
    }
}
