#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use core::result::Result;

use std::prelude::rust_2024::*;

use std::io;

use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use File_system::{
    File_system_traits, Flags_type, Mode_type, Open_type, Path_owned_type, Path_type, Time_type,
};

use Users::{Group_identifier_type, User_identifier_type};

use Task::Task_identifier_type;

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug)]
pub enum Error_type {
    File_system_error(File_system::Error_type),
    Io_error(io::Error),
}

impl From<File_system::Error_type> for Error_type {
    fn from(Error: File_system::Error_type) -> Self {
        Self::File_system_error(Error)
    }
}

impl From<io::Error> for Error_type {
    fn from(Error: io::Error) -> Self {
        Self::Io_error(Error)
    }
}

pub struct Loader_type {
    Paths: Vec<(PathBuf, Path_owned_type)>,
}

impl Loader_type {
    pub fn New() -> Self {
        Self { Paths: Vec::new() }
    }

    pub fn Add_file(
        mut self,
        Source: impl AsRef<Path>,
        Destination: impl AsRef<Path_type>,
    ) -> Self {
        self.Paths
            .push((Source.as_ref().to_owned(), Destination.as_ref().to_owned()));

        self
    }

    pub fn Add_files(
        mut self,
        Files: impl IntoIterator<Item = (PathBuf, Path_owned_type)>,
    ) -> Self {
        for File in Files {
            self = self.Add_file(File.0, File.1);
        }

        self
    }

    pub fn Load(&self, File_system: &mut dyn File_system_traits) -> Result_type<()> {
        // Open file for reading on host
        for (Source_path, Destination_path) in &self.Paths {
            // Open file for reading on host
            let mut Source_file = File::open(Source_path)?;

            // Create file on target
            let Destination_file = File_system.Open(
                Task_identifier_type::New(0),
                Destination_path,
                Flags_type::New(Mode_type::Read_only, Some(Open_type::Create), None),
                Time_type::New(0),
                User_identifier_type::Root,
                Group_identifier_type::Root,
            )?;

            // Read and write file content block by block
            let mut Buffer = [0; 1024];
            loop {
                let Read = Source_file.read(&mut Buffer)?;

                if Read == 0 {
                    break;
                }

                File_system.Write(Destination_file, &Buffer[..Read], Time_type::New(0))?;
            }

            File_system.Close(Destination_file)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod Tests {
    use super::*;
    use File_system::File_system_traits;

    #[test]
    fn Test_loader() {
        // - Load the file in the file system
        let Source_path = "Cargo.toml";
        let Destination_path = "/Cargo.toml";

        let Device = File_system::Create_device!(File_system::Memory_device_type::<512>::New(
            1024 * 1024 * 512
        ));

        LittleFS::File_system_type::Format(Device.clone(), 256).unwrap();
        let mut File_system = LittleFS::File_system_type::New(Device, 256).unwrap();

        let Loader = Loader_type::New().Add_file(Source_path, Destination_path);

        Loader.Load(&mut File_system).unwrap();

        // - Read the file and compare it with the original
        let Test_file = std::fs::read_to_string(Source_path).unwrap();

        let mut Buffer = vec![0; Test_file.len()];

        let File = File_system
            .Open(
                Task_identifier_type::New(0),
                Path_type::New(Destination_path),
                Flags_type::New(Mode_type::Read_only, None, None),
                Time_type::New(0),
                User_identifier_type::Root,
                Group_identifier_type::Root,
            )
            .unwrap();

        let Read = File_system
            .Read(File, &mut Buffer, Time_type::New(0))
            .unwrap();

        assert_eq!(Read, Test_file.len());
        assert_eq!(Buffer, Test_file.as_bytes());
    }
}
