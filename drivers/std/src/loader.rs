use file_system::{AccessFlags, CreateFlags, Flags, Path};
use std::io;
use std::prelude::rust_2024::*;
use std::{fs::File, io::Read, path};
use virtual_file_system::VirtualFileSystem;

#[derive(Debug)]
pub enum Error {
    VirtualFileSystemError(virtual_file_system::Error),
    IoError(io::Error),
}

impl From<virtual_file_system::Error> for Error {
    fn from(error: virtual_file_system::Error) -> Self {
        Error::VirtualFileSystemError(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

pub type Result<T> = core::result::Result<T, Error>;

pub async fn load_to_virtual_file_system<'a>(
    virtual_file_system: &'a VirtualFileSystem<'a>,
    source_path: impl AsRef<path::Path>,
    destination_path: impl AsRef<Path>,
) -> Result<()> {
    // Open file for reading on host
    let mut source_file = File::open(source_path.as_ref())?;

    let task = task::get_instance().get_current_task_identifier().await;

    log::information!(
        "Loading file to virtual file system at path: {:?}",
        destination_path.as_ref()
    );

    let mut file = virtual_file_system::File::open(
        virtual_file_system,
        task,
        &destination_path.as_ref(),
        Flags::new(AccessFlags::Write, Some(CreateFlags::CREATE_TRUNCATE), None),
    )
    .await?;

    log::information!(
        "Copying content to virtual file system at path: {:?}",
        destination_path.as_ref()
    );

    // Read and write file content block by block
    let mut buffer = [0; 512];
    let mut total_written = 0;
    loop {
        let bytes_read = source_file.read(&mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        if total_written > 152 * 1024 {
            log::information!(
                "Written {} KB to virtual file system at path: {:?}",
                total_written / 1024,
                destination_path.as_ref()
            );
        }

        file.write(&buffer[..bytes_read]).await?;
        total_written += bytes_read;
    }

    log::information!(
        "Finished loading file to virtual file system at path: {:?}, total bytes: {}",
        destination_path.as_ref(),
        total_written
    );

    file.close(virtual_file_system).await?;

    log::information!(
        "Closed file in virtual file system at path: {:?}",
        destination_path.as_ref()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use task::test;

    #[test(executor = crate::executor::Executor::new_static())]
    async fn test_loader() {
        // - Load the file in the file system
        let source_path = "Cargo.toml";
        let destination_path = "/Cargo.toml";

        let device = file_system::MemoryDevice::<512>::new_static(1024 * 1024 * 512);

        let task_instance = task::initialize();

        let _ = users::initialize();
        let _ = time::initialize(&crate::devices::TimeDevice);

        let task = task_instance.get_current_task_identifier().await;

        little_fs::FileSystem::format(device, 256).unwrap();
        let file_system = little_fs::FileSystem::new(device, 256).unwrap();

        let virtual_file_system = virtual_file_system::initialize(
            task::get_instance(),
            users::get_instance(),
            time::get_instance(),
            file_system,
            None,
        )
        .unwrap();

        load_to_virtual_file_system(virtual_file_system, source_path, destination_path)
            .await
            .unwrap();

        // - Read the file and compare it with the original
        let test_file = std::fs::read_to_string(source_path).unwrap();

        let mut buffer = vec![0; test_file.len()];

        virtual_file_system::File::read_from_path(
            virtual_file_system,
            task,
            &destination_path,
            &mut buffer,
        )
        .await
        .unwrap();

        assert_eq!(buffer, test_file.as_bytes());
    }
}
