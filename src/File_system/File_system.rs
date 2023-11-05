use super::File::*;

pub trait File_system_traits {
    type File_type : File_traits;

    fn Initialize(&mut self) -> Result<(), ()>;

    // - Status
    fn Exists(&self, Path : &str) -> Result<bool, ()>;
    
    // - Manipulation
    // - - File
    fn Open_file(&self, Path : &str, Mode : Mode_type) -> Result<Self::File_type, ()>;
    fn Delete_file(&self, Path : &str) -> Result<(), ()>;
    // - - Directory
    fn Create_directory(&self, Path : &str) -> Result<(), ()>;
    fn Create_directory_recursive(&self, Path : &str) -> Result<(), ()>;
    fn Delete_directory(&self, Path : &str) -> Result<(), ()>;   
    fn Delete_directory_recursive(&self, Path : &str) -> Result<(), ()>;
    fn Move(&self, Path : &str, Destination : &str) -> Result<(), ()>;
}
