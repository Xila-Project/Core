use file_system::{FileSystemTraits, Flags, Mode, Open, Path, PathOwned, Time};
use std::io;
use std::prelude::rust_2024::*;
use std::{fs::File, io::Read, path};
use task::TaskIdentifier;
use users::{GroupIdentifier, UserIdentifier};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FileSystemError(file_system::Error),
    IoError(io::Error),
}

impl From<file_system::Error> for Error {
    fn from(error: file_system::Error) -> Self {
        Self::FileSystemError(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

pub struct Loader<'a> {
    paths: Vec<(path::PathBuf, PathOwned)>,
    buffers: Vec<(&'a [u8], PathOwned)>,
}

impl Default for Loader<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Loader<'a> {
    pub fn new() -> Self {
        Self {
            paths: vec![],
            buffers: vec![],
        }
    }

    pub fn add_file_from_buffer(mut self, buffer: &'a [u8], destination: impl AsRef<Path>) -> Self {
        self.buffers.push((buffer, destination.as_ref().to_owned()));

        self
    }

    pub fn add_files_from_buffers(
        mut self,
        buffers: impl IntoIterator<Item = (&'a [u8], PathOwned)>,
    ) -> Self {
        for (buffer, destination) in buffers {
            self = self.add_file_from_buffer(buffer, destination);
        }

        self
    }

    pub fn add_file(
        mut self,
        source: impl AsRef<path::Path>,
        destination: impl AsRef<Path>,
    ) -> Self {
        self.paths
            .push((source.as_ref().to_owned(), destination.as_ref().to_owned()));

        self
    }

    pub fn add_files(
        mut self,
        files: impl IntoIterator<Item = (path::PathBuf, PathOwned)>,
    ) -> Self {
        for file in files {
            self = self.add_file(file.0, file.1);
        }

        self
    }

    pub fn load(&self, file_system: &mut dyn FileSystemTraits) -> Result<()> {
        // Open file for reading on host
        for (source_path, destination_path) in &self.paths {
            // Open file for reading on host
            let mut source_file = File::open(source_path)?;

            // Create file on target
            let destination_file = file_system.open(
                TaskIdentifier::new(0),
                destination_path,
                Flags::new(Mode::READ_ONLY, Some(Open::CREATE), None),
                Time::new(0),
                UserIdentifier::ROOT,
                GroupIdentifier::ROOT,
            )?;

            // Read and write file content block by block
            let mut buffer = [0; 1024];
            loop {
                let read = source_file.read(&mut buffer)?;

                if read == 0 {
                    break;
                }

                file_system.write(destination_file, &buffer[..read], Time::new(0))?;
            }

            file_system.close(destination_file)?;
        }

        // Write buffers to file system
        for (buffer, destination_path) in &self.buffers {
            let destination_file = file_system.open(
                TaskIdentifier::new(0),
                destination_path,
                Flags::new(Mode::READ_ONLY, Some(Open::CREATE), None),
                Time::new(0),
                UserIdentifier::ROOT,
                GroupIdentifier::ROOT,
            )?;

            file_system.write(destination_file, buffer, Time::new(0))?;
            file_system.close(destination_file)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use file_system::FileSystemTraits;

    #[test]
    fn test_loader() {
        // - Load the file in the file system
        let source_path = "Cargo.toml";
        let destination_path = "/Cargo.toml";

        let device =
            file_system::create_device!(file_system::MemoryDevice::<512>::new(1024 * 1024 * 512));

        little_fs::FileSystem::format(device.clone(), 256).unwrap();
        let mut file_system = little_fs::FileSystem::new(device, 256).unwrap();

        let loader = Loader::new().add_file(source_path, destination_path);

        loader.load(&mut file_system).unwrap();

        // - Read the file and compare it with the original
        let test_file = std::fs::read_to_string(source_path).unwrap();

        let mut buffer = vec![0; test_file.len()];

        let file: file_system::LocalFileIdentifier = file_system
            .open(
                TaskIdentifier::new(0),
                Path::new(destination_path),
                Flags::new(Mode::READ_ONLY, None, None),
                Time::new(0),
                UserIdentifier::ROOT,
                GroupIdentifier::ROOT,
            )
            .unwrap();

        let read = file_system.read(file, &mut buffer, Time::new(0)).unwrap();

        assert_eq!(read, test_file.len());
        assert_eq!(buffer, test_file.as_bytes());
    }
}
