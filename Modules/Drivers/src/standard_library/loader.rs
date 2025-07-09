#![allow(non_camel_case_types)]

use core::result::Result;

use std::prelude::rust_2024::*;

use std::io;

use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use file_system::{
    File_system_traits, Flags_type, Mode_type, Open_type, Path_owned_type, Path_type, Time_type,
};

use users::{Group_identifier_type, User_identifier_type};

use task::Task_identifier_type;

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug)]
pub enum Error_type {
    File_system_error(file_system::Error_type),
    Io_error(io::Error),
}

impl From<file_system::Error_type> for Error_type {
    fn from(error: file_system::Error_type) -> Self {
        Self::File_system_error(error)
    }
}

impl From<io::Error> for Error_type {
    fn from(error: io::Error) -> Self {
        Self::Io_error(error)
    }
}

pub struct Loader_type {
    paths: Vec<(PathBuf, Path_owned_type)>,
}

impl Loader_type {
    pub fn new() -> Self {
        Self { paths: Vec::new() }
    }

    pub fn add_file(
        mut self,
        source: impl AsRef<Path>,
        destination: impl AsRef<Path_type>,
    ) -> Self {
        self.paths
            .push((source.as_ref().to_owned(), destination.as_ref().to_owned()));

        self
    }

    pub fn add_files(
        mut self,
        files: impl IntoIterator<Item = (PathBuf, Path_owned_type)>,
    ) -> Self {
        for file in files {
            self = self.add_file(file.0, file.1);
        }

        self
    }

    pub fn load(&self, file_system: &mut dyn File_system_traits) -> Result_type<()> {
        // Open file for reading on host
        for (source_path, destination_path) in &self.paths {
            // Open file for reading on host
            let mut source_file = File::open(source_path)?;

            // Create file on target
            let destination_file = file_system.open(
                Task_identifier_type::new(0),
                destination_path,
                Flags_type::New(Mode_type::READ_ONLY, Some(Open_type::CREATE), None),
                Time_type::new(0),
                User_identifier_type::ROOT,
                Group_identifier_type::ROOT,
            )?;

            // Read and write file content block by block
            let mut buffer = [0; 1024];
            loop {
                let read = source_file.read(&mut buffer)?;

                if read == 0 {
                    break;
                }

                file_system.write(destination_file, &buffer[..read], Time_type::new(0))?;
            }

            file_system.close(destination_file)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use file_system::File_system_traits;

    #[test]
    fn test_loader() {
        // - Load the file in the file system
        let source_path = "Cargo.toml";
        let destination_path = "/Cargo.toml";

        let device = file_system::Create_device!(file_system::Memory_device_type::<512>::New(
            1024 * 1024 * 512
        ));

        little_fs::File_system_type::format(device.clone(), 256).unwrap();
        let mut file_system = little_fs::File_system_type::new(device, 256).unwrap();

        let loader = Loader_type::new().add_file(source_path, destination_path);

        loader.load(&mut file_system).unwrap();

        // - Read the file and compare it with the original
        let test_file = std::fs::read_to_string(source_path).unwrap();

        let mut buffer = vec![0; test_file.len()];

        let file: file_system::Local_file_identifier_type = file_system
            .open(
                Task_identifier_type::new(0),
                Path_type::New(destination_path),
                Flags_type::New(Mode_type::READ_ONLY, None, None),
                Time_type::new(0),
                User_identifier_type::ROOT,
                Group_identifier_type::ROOT,
            )
            .unwrap();

        let read = file_system
            .read(file, &mut buffer, Time_type::new(0))
            .unwrap();

        assert_eq!(read, test_file.len());
        assert_eq!(buffer, test_file.as_bytes());
    }
}
