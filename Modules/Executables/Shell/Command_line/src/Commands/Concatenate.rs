use File_system::{Mode_type, Path_type};

use crate::Shell_type;

impl Shell_type {
    pub fn Concatenate(&mut self, Arguments: &[&str]) {
        for Path in Arguments {
            let File = match Virtual_file_system::Get_instance().Open(
                &Path_type::From_str(Path),
                Mode_type::Read_only.into(),
                self.Standard.Get_task(),
            ) {
                Ok(File) => File,
                Err(Error) => {
                    self.Standard
                        .Print_error_line(&format!("Failed to open file: {:?}", Error));
                    continue;
                }
            };

            let mut Buffer = [0_u8; 128];
            while let Ok(Size) = Virtual_file_system::Get_instance().Read(
                File,
                &mut Buffer,
                self.Standard.Get_task(),
            ) {
                if Size == 0 {
                    break;
                }

                let Size: usize = Size.into();

                self.Standard.Write(&Buffer[..Size]);
            }
        }
    }
}
