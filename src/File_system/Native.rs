use super::{File::*, File_system::File_system_traits, Fundamentals::*};

use std::env::{current_dir, var};
use std::fs::*;
use std::io::{Read, Seek, Write};
use std::path::Path;

pub struct Native_file_system_type {
    Virtual_root_path: String,
}

impl Native_file_system_type {
    pub fn New() -> Self {
        Native_file_system_type {
            Virtual_root_path: String::new(),
        }
    }

    pub fn Get_full_path(&self, Path: &str) -> String {
        let mut Full_path = self.Virtual_root_path.clone();
        Full_path.reserve(Path.len() + 1);
        Full_path.push(std::path::MAIN_SEPARATOR);
        Full_path.push_str(Path);
        Full_path
    }
}

impl File_system_traits for Native_file_system_type {
    type File_type = Native_file_type;

    fn Initialize(&mut self) -> Result<(), ()> {
        match var("Xila_virtual_root_path") {
            Ok(value) => {
                self.Virtual_root_path = value;
            }
            Err(_) => match current_dir() {
                Ok(value) => {
                    self.Virtual_root_path = value.to_str().unwrap().to_string();
                }
                Err(_) => {
                    return Err(());
                }
            },
        }
        self.Virtual_root_path.push(std::path::MAIN_SEPARATOR);
        self.Virtual_root_path += "Xila";

        if !Path::new(&self.Virtual_root_path).exists() {
            match create_dir(&self.Virtual_root_path) {
                Ok(_) => {}
                Err(_) => {
                    return Err(());
                }
            }
        }

        Ok(())
    }

    fn Exists(&self, Path: &str) -> Result<bool, ()> {
        Path::new(&self.Get_full_path(Path))
            .try_exists()
            .map_err(|_| ())
    }

    fn Open_file(&self, Path: &str, Mode: Mode_type) -> Result<Self::File_type, ()> {
        let Full_path = self.Get_full_path(Path);
        match Mode {
            Mode_type::Read => match File::open(&Full_path) {
                Ok(Data) => Ok(Native_file_type(Data)),
                Err(_) => Err(()),
            },
            Mode_type::Write => match File::create(&Full_path) {
                Ok(Data) => Ok(Native_file_type(Data)),
                Err(_) => Err(()),
            },
            Mode_type::Read_write => {
                match OpenOptions::new().read(true).write(true).open(&Full_path) {
                    Ok(Data) => Ok(Native_file_type(Data)),
                    Err(_) => Err(()),
                }
            }
            Mode_type::Append => match OpenOptions::new().append(true).open(&Full_path) {
                Ok(Data) => Ok(Native_file_type(Data)),
                Err(_) => Err(()),
            },
            Mode_type::Read_append => {
                match OpenOptions::new().read(true).append(true).open(&Full_path) {
                    Ok(Data) => Ok(Native_file_type(Data)),
                    Err(_) => Err(()),
                }
            }
        }
    }

    fn Delete_file(&self, Path: &str) -> Result<(), ()> {
        remove_file(self.Get_full_path(Path)).map_err(|_| ())
    }

    fn Create_directory(&self, Path: &str) -> Result<(), ()> {
        create_dir(self.Get_full_path(Path)).map_err(|_| ())
    }

    fn Create_directory_recursive(&self, Path: &str) -> Result<(), ()> {
        create_dir_all(self.Get_full_path(Path)).map_err(|_| ())
    }

    fn Delete_directory(&self, Path: &str) -> Result<(), ()> {
        remove_dir(self.Get_full_path(Path)).map_err(|_| ())
    }

    fn Delete_directory_recursive(&self, Path: &str) -> Result<(), ()> {
        remove_dir_all(self.Get_full_path(Path)).map_err(|_| ())
    }

    fn Move(&self, Path: &str, Destination: &str) -> Result<(), ()> {
        rename(self.Get_full_path(Path), &self.Get_full_path(Destination)).map_err(|_| ())
    }
}

// - File

pub struct Native_file_type(std::fs::File);

impl Read for Native_file_type {
    fn read(&mut self, Buffer: &mut [u8]) -> Result<usize, std::io::Error> {
        self.0.read(Buffer)
    }
}

impl Write for Native_file_type {
    fn write(&mut self, Buffer: &[u8]) -> Result<usize, std::io::Error> {
        self.0.write(Buffer)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.0.flush()
    }
}

impl Seek for Native_file_type {
    fn seek(&mut self, Position: std::io::SeekFrom) -> Result<u64, std::io::Error> {
        self.0.seek(Position)
    }
}

impl From<FileType> for Type_type {
    fn from(value: FileType) -> Self {
        if value.is_dir() {
            return Type_type::Directory;
        } else if value.is_symlink() {
            return Type_type::Symbolic_link;
        }
        Type_type::File
    }
}

impl File_traits for Native_file_type {
    fn Get_size(&self) -> Result<Size_type, ()> {
        match self.0.metadata() {
            Ok(metadata) => Ok(metadata.len().into()),
            Err(_) => Err(()),
        }
    }

    fn Get_type(&self) -> Result<Type_type, ()> {
        match self.0.metadata() {
            Ok(metadata) => Ok(metadata.file_type().into()),
            Err(_) => Err(()),
        }
    }
}

// - Test
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    const Test_directory_path: &str = "Test";

    fn Get_path_in_test(Path: &str) -> String {
        Test_directory_path.to_string() + std::path::MAIN_SEPARATOR.to_string().as_str() + Path
    }

    fn Create_test_directory(File_system: &Native_file_system_type) {
        let mut Test_path = Test_directory_path.to_string();
        Test_path = File_system.Get_full_path(Test_path.as_str());
        if !Path::new(&Test_path).exists() {
            create_dir(&Test_path).unwrap();
        }
    }

    #[test]
    fn Exists() {
        let mut File_system = Native_file_system_type::New();
        assert!(File_system.Initialize().is_ok());
        Create_test_directory(&File_system);
        let File_path_string = Get_path_in_test("exists.txt");
        let File_path = File_path_string.as_str();
        assert!(!File_system.Exists(File_path).unwrap());
        let mut File = File::create(File_system.Get_full_path(File_path)).unwrap();
        assert!(File.write_all(b"Hello, world!").is_ok());
        assert!(File_system.Exists(File_path).unwrap());
        assert!(remove_file(File_system.Get_full_path(File_path)).is_ok());
        assert!(!File_system.Exists(File_path).unwrap());
    }

    #[test]
    fn File_manipulation() {
        let mut File_system = Native_file_system_type::New();
        assert!(File_system.Initialize().is_ok());
        Create_test_directory(&File_system);

        let File_path_string = Get_path_in_test("delete_file.txt");
        let File_path = File_path_string.as_str();
        assert!(!File_system.Exists(File_path).unwrap());
        assert!(File_system.Open_file(File_path, Mode_type::Write).is_ok());
        assert!(File_system.Exists(File_path).unwrap());
        assert!(File_system.Delete_file(File_path).is_ok());
        assert!(!File_system.Exists(File_path).unwrap());
    }

    #[test]
    fn Directory_operations() {
        let mut File_system = Native_file_system_type::New();
        assert!(File_system.Initialize().is_ok());

        let Directory_path_string = Get_path_in_test("directory");
        let Directory_path = Directory_path_string.as_str();
        assert!(!File_system.Exists(Directory_path).unwrap());
        assert!(File_system.Create_directory(Directory_path).is_ok());
        assert!(File_system.Exists(Directory_path).unwrap());
        assert!(File_system.Delete_directory(Directory_path).is_ok());
        assert!(!File_system.Exists(Directory_path).unwrap());
    }

    #[test]
    fn File_operations() {
        let mut File_system = Native_file_system_type::New();
        assert!(File_system.Initialize().is_ok());
        Create_test_directory(&File_system);

        let File_path_string = Get_path_in_test("file_operations.txt");
        let File_path = File_path_string.as_str();
        assert!(!File_system.Exists(File_path).unwrap());
        let mut File = File_system.Open_file(File_path, Mode_type::Write).unwrap();
        assert!(File.write_all(b"Hello, world!").is_ok());
        assert!(File_system.Exists(File_path).unwrap());
        assert!(File_system.Delete_file(File_path).is_ok());
        assert!(!File_system.Exists(File_path).unwrap());
    }

    #[test]
    fn File_metadata() {
        let mut File_system = Native_file_system_type::New();
        assert!(File_system.Initialize().is_ok());
        Create_test_directory(&File_system);

        let File_path_string = Get_path_in_test("file_metadata.txt");
        let File_path = File_path_string.as_str();
        assert!(!File_system.Exists(File_path).unwrap());
        let mut File = File_system.Open_file(File_path, Mode_type::Write).unwrap();
        assert!(File.write_all(b"Hello, world!").is_ok());
        assert!(File_system.Exists(File_path).unwrap());
        assert!(File.Get_size().unwrap() == Size_type(13));
        assert!(File_system.Delete_file(File_path).is_ok());
        assert!(!File_system.Exists(File_path).unwrap());
    }
}
