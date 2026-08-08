use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use xila::{
    file_system::PathOwned,
    task::TaskIdentifier,
    virtual_file_system::{SynchronousDirectory, SynchronousFile},
};

pub struct WasiContext {
    pub files: BTreeMap<u32, FileSystemItem>,
    pub args: Vec<Vec<u8>>,
    pub task: TaskIdentifier,
    pub random_state: u64,
    pub prestats: Vec<Prestat>,
    pub exit_code: Option<i32>,
}

impl WasiContext {
    pub fn get_file_system_item(&self, fd: u32) -> Option<&FileSystemItem> {
        self.files.get(&fd)
    }

    pub fn get_synchronous_file(&self, fd: u32) -> Option<&SynchronousFile> {
        self.files
            .get(&fd)
            .and_then(|item| item.into_synchronous_file())
    }

    pub fn get_synchronous_directory(&self, fd: u32) -> Option<&SynchronousDirectory> {
        self.files
            .get(&fd)
            .and_then(|item| item.into_synchronous_directory())
    }
}

pub struct Prestat {
    pub name: Vec<u8>,
}

pub struct FileVariant {
    pub file: SynchronousFile,
}

pub struct DirectoryVariant {
    pub path: PathOwned,
    pub directory: SynchronousDirectory,
}

pub enum FileSystemItem {
    StandardInput(FileVariant),
    StandardOutput(FileVariant),
    StandardError(FileVariant),
    File(FileVariant),
    Directory(DirectoryVariant),
}

impl FileSystemItem {
    pub fn into_synchronous_file(&self) -> Option<&SynchronousFile> {
        match self {
            FileSystemItem::StandardInput(file) => Some(&file.file),
            FileSystemItem::StandardOutput(file) => Some(&file.file),
            FileSystemItem::StandardError(file) => Some(&file.file),
            FileSystemItem::File(file) => Some(&file.file),
            FileSystemItem::Directory(_) => None,
        }
    }

    pub fn into_synchronous_directory(&self) -> Option<&SynchronousDirectory> {
        match self {
            FileSystemItem::Directory(dir) => Some(&dir.directory),
            _ => None,
        }
    }
}
