use file_system::{FileSystemTraits, Flags, Mode, Open, Path, Time};
use std::io;
use std::prelude::rust_2024::*;
use std::{fs::File, io::Read, path};
use task::TaskIdentifier;
use users::{GroupIdentifier, UserIdentifier};
use virtual_file_system::VirtualFileSystem;

#[derive(Debug)]
pub enum Error {
    FileSystemError(file_system::Error),
    IoError(io::Error),
}

impl From<file_system::Error> for Error {
    fn from(error: file_system::Error) -> Self {
        Error::FileSystemError(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

pub type Result<T> = core::result::Result<T, Error>;

pub fn load_to_file_system(
    file_system: &mut dyn FileSystemTraits,
    source_path: impl AsRef<path::Path>,
    destination_path: impl AsRef<Path>,
) -> Result<()> {
    // Open file for reading on host
    let mut source_file = File::open(source_path.as_ref())?;

    // Create file on target
    let destination_file = file_system.open(
        TaskIdentifier::new(0),
        destination_path.as_ref(),
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

    Ok(())
}

pub async fn load_to_virtual_file_system<'a>(
    virtual_file_system: &'a VirtualFileSystem<'a>,
    source_path: impl AsRef<path::Path>,
    destination_path: impl AsRef<Path>,
) -> Result<()> {
    // Open file for reading on host
    let mut source_file = File::open(source_path.as_ref())?;

    let file = virtual_file_system::File::open(
        virtual_file_system,
        &destination_path.as_ref(),
        Flags::new(Mode::READ_ONLY, Some(Open::CREATE), None),
    )
    .await?;

    // Read and write file content block by block
    let mut buffer = [0; 1024];
    loop {
        let read = source_file.read(&mut buffer)?;

        if read == 0 {
            break;
        }

        file.write(&buffer[..read]).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use file_system::FileSystemTraits;
    extern crate alloc;

    #[test]
    fn test_loader() {
        // - Load the file in the file system
        let source_path = "Cargo.toml";
        let destination_path = "/Cargo.toml";

        let device =
            file_system::create_device!(file_system::MemoryDevice::<512>::new(1024 * 1024 * 512));

        little_fs::FileSystem::format(device.clone(), 256).unwrap();
        let mut file_system = little_fs::FileSystem::new(device, 256).unwrap();

        load_to_file_system(&mut file_system, source_path, destination_path).unwrap();

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
