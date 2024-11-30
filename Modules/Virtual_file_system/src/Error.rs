#[derive(Debug)]
pub enum Error_type {
    Already_initialized,
    File_system(File_system::Error_type),
}

impl From<File_system::Error_type> for Error_type {
    fn from(Value: File_system::Error_type) -> Self {
        Self::File_system(Value)
    }
}
