use crate::File_system::Prelude::Path_type;

use super::File::*;

pub trait File_system_traits {
    type File_type: File_traits;

    fn Initialize(&mut self) -> Result<(), ()>;

    // fn Mount_file_system<F : File_system_traits>(&mut self, File_system : &mut F, Mount_point : &str) -> Result<(), ()>;

    // - Status
    fn Exists(&self, Path: &Path_type) -> Result<bool, ()>;

    // - Manipulation
    // - - File
    fn Open_file(&self, Path: &Path_type, Mode: Mode_type) -> Result<Self::File_type, ()>;
    fn Delete_file(&self, Path: &Path_type) -> Result<(), ()>;
    // - - Directory
    fn Create_directory(&self, Path: &Path_type) -> Result<(), ()>;
    fn Create_directory_recursive(&self, Path: &Path_type) -> Result<(), ()>;
    fn Delete_directory(&self, Path: &Path_type) -> Result<(), ()>;
    fn Delete_directory_recursive(&self, Path: &Path_type) -> Result<(), ()>;
    fn Move(&self, Path: &Path_type, Destination: &Path_type) -> Result<(), ()>;
}
