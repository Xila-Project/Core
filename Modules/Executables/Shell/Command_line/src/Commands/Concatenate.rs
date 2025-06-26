use alloc::format;
use File_system::{Mode_type, Path_type};

use crate::Shell_type;

impl Shell_type {
    async fn Read_file_and_write(&mut self, Path: &Path_type) {
        let File = match Virtual_file_system::Get_instance()
            .Open(&Path, Mode_type::Read_only.into(), self.Standard.Get_task())
            .await
        {
            Ok(File) => File,
            Err(Error) => {
                self.Standard
                    .Print_error_line(&format!("Failed to open file: {:?}", Error))
                    .await;
                return;
            }
        };

        let mut Buffer = [0_u8; 128];
        while let Ok(Size) = Virtual_file_system::Get_instance()
            .Read(File, &mut Buffer, self.Standard.Get_task())
            .await
        {
            if Size == 0 {
                break;
            }

            let Size: usize = Size.into();

            self.Standard.Write(&Buffer[..Size]).await;
        }
    }

    pub async fn Concatenate(&mut self, Arguments: &[&str]) {
        for Path in Arguments {
            let Path = Path_type::From_str(Path);

            if Path.Is_absolute() {
                self.Read_file_and_write(Path).await;
            } else {
                match self.Current_directory.clone().Join(Path) {
                    Some(Path) => self.Read_file_and_write(&Path).await,
                    None => self.Standard.Print_error_line("Invalid command").await,
                }
            }
        }
    }
}
