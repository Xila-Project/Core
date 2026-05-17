use core::ptr::NonNull;

use alloc::boxed::Box;
use smol_str::SmolStr;
use xila::abi_declarations::{XilaFileSystemDirectory, XilaFileSystemFile};
use xila::shared::generate_shadow_type;
use xila::{
    file_system::{Path, PathOwned},
    log,
};

pub enum FileVariantKind {
    Regular,
    StandardInput,
    StandardOutput,
    StandardError,
}

pub struct FileVariant {
    pub file: XilaFileSystemFile,
    pub kind: FileVariantKind,
}

pub struct DirectoryVariant {
    pub name: SmolStr,
    pub parent: Option<NonNull<FileSystemItem>>,
    pub directory: XilaFileSystemDirectory,
}

pub enum FileSystemItem {
    File(FileVariant),
    Directory(DirectoryVariant),
}

impl FileSystemItem {
    pub fn new_file(file: XilaFileSystemFile, kind: FileVariantKind) -> *mut Self {
        Box::into_raw(Box::new(FileSystemItem::File(FileVariant { file, kind })))
    }

    pub fn new_directory(
        directory: XilaFileSystemDirectory,
        parent: Option<NonNull<FileSystemItem>>,
        path: &str,
    ) -> *mut Self {
        let item = FileSystemItem::Directory(DirectoryVariant {
            name: path.into(),
            parent,
            directory,
        });

        Box::into_raw(Box::new(item))
    }

    pub unsafe fn borrow_from_raw<'a>(ptr: *mut Self) -> &'a mut Self {
        unsafe { &mut *ptr }
    }

    pub unsafe fn own_from_raw(ptr: *mut Self) -> Box<Self> {
        unsafe { Box::from_raw(ptr) }
    }

    pub fn as_file(&self) -> Option<&FileVariant> {
        match self {
            FileSystemItem::File(file) => Some(file),
            _ => None,
        }
    }

    pub fn as_directory(&self) -> Option<&DirectoryVariant> {
        match self {
            FileSystemItem::Directory(directory) => Some(directory),
            _ => None,
        }
    }

    pub fn resolve(&self, path: impl AsRef<Path>) -> Option<PathOwned> {
        let mut total_size = path.as_ref().get_length();

        // 1. Iterate over to reach the root and calculate the total size of the path
        let mut current = Some(NonNull::from(self));
        while let Some(item) = current {
            match unsafe { item.read() } {
                FileSystemItem::File(file) => {
                    log::warning!(
                        "Attempting to resolve path {:?} through a file: {:p}",
                        path.as_ref(),
                        &file
                    );
                    return None; // Files cannot be resolved further
                }
                FileSystemItem::Directory(directory) => {
                    log::information!(
                        "Directory: {}, Parent: {:?}",
                        directory.name,
                        directory.parent
                    );
                    total_size += directory.name.len() + 1; // +1 for the separator
                    current = directory.parent;
                }
            }
        }

        // 2. Allocate memory for the resolved path
        let mut resolved_path = PathOwned::new_with_capacity(total_size);

        // 3. Iterate over to build the resolved path
        let mut current = Some(NonNull::from(self));
        while let Some(item) = current {
            match unsafe { item.read() } {
                FileSystemItem::File(file) => {
                    log::warning!(
                        "Attempting to resolve path {:?} through a file: {:p}",
                        path.as_ref(),
                        &file
                    );
                    return None; // Files cannot be resolved further
                }
                FileSystemItem::Directory(directory) => {
                    log::information!(
                        "Directory: {}, Parent: {:?}",
                        directory.name,
                        directory.parent
                    );
                    resolved_path = resolved_path.join(&directory.name.as_str())?;
                    current = directory.parent;
                }
            }
        }

        resolved_path = resolved_path.join(path.as_ref())?;
        resolved_path = resolved_path.join("\0")?; // Null terminator for C compatibility

        Some(resolved_path)
    }
}

generate_shadow_type!(XilaFileSystemItem, FileSystemItem, 40, 8);
