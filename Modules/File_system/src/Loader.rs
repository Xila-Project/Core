#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use core::result::Result;
use std::io;

use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use crate::{
    File_system_traits, Flags_type, Mode_type, Open_type, Path_owned_type, Path_type, Time_type,
};

use Task::Task_identifier_type;
use Users::{Group_identifier_type, User_identifier_type};

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug)]
pub enum Error_type {
    File_system_error(crate::Error_type),
    Io_error(io::Error),
}

impl From<crate::Error_type> for Error_type {
    fn from(Error: crate::Error_type) -> Self {
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
