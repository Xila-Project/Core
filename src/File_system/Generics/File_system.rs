use super::*;

pub trait File_system_traits {
    fn Initialize(&mut self) -> Result<(), ()>;

    // - Status
    fn Exists(&self, Path: &Path_type) -> Result<bool, ()>;

    // - Mount points
    //fn Mount(&mut self, File_system: &mut Self, Mount_point: &Path_type) -> Result<(), ()>;
    //fn Unmount(&mut self, Mount_point: &Path_type) -> Result<(), ()>;

    // - Manipulation
    // - - File operations
    fn Open_file(&self, Path: &Path_type, Mode: Mode_type) -> Result<File_type, ()>;
    fn Close_file(&self, File: File_identifier_type) -> Result<(), ()>;
    fn Read_file(&self, File: File_identifier_type, Buffer: &mut [u8]) -> Result<usize, ()>;
    fn Write_file(&self, File: File_identifier_type, Buffer: &[u8]) -> Result<usize, ()>;
    fn Flush_file(&self, File: File_identifier_type) -> Result<(), ()>;
    fn Get_file_type(&self, File: File_identifier_type) -> Result<Type_type, ()>;
    fn Get_file_size(&self, File: File_identifier_type) -> Result<Size_type, ()>;
    fn Get_file_position(&self, File: File_identifier_type) -> Result<Size_type, ()>;
    fn Set_file_position(
        &self,
        File: File_identifier_type,
        Position: Position_type,
    ) -> Result<Size_type, ()>;

    fn Delete_file(&self, Path: &Path_type) -> Result<(), ()>;
    // - - Directory
    fn Create_directory(&self, Path: &Path_type) -> Result<(), ()>;
    fn Create_directory_recursive(&self, Path: &Path_type) -> Result<(), ()>;
    fn Delete_directory(&self, Path: &Path_type) -> Result<(), ()>;
    fn Delete_directory_recursive(&self, Path: &Path_type) -> Result<(), ()>;
    fn Move(&self, Path: &Path_type, Destination: &Path_type) -> Result<(), ()>;
}
