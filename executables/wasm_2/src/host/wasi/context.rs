use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};
use xila::{
    file_system::PathOwned,
    task::TaskIdentifier,
    virtual_file_system::{SynchronousDirectory, SynchronousFile},
};

use crate::host::translation::WasmPod;

pub struct WasiContext {
    pub files: BTreeMap<u32, FileSystemItem>,
    pub arguments: Vec<String>,
    pub task: TaskIdentifier,
    pub random_state: u64,
    pub prestats: Vec<Prestat>,
    pub exit_code: Option<i32>,
}

impl WasiContext {
    pub fn get_file_system_item(&mut self, fd: u32) -> Option<&mut FileSystemItem> {
        self.files.get_mut(&fd)
    }

    pub fn get_synchronous_file(&mut self, fd: u32) -> Option<&mut SynchronousFile> {
        self.files
            .get_mut(&fd)
            .and_then(|item| item.into_synchronous_file())
    }

    pub fn get_synchronous_directory(&mut self, fd: u32) -> Option<&mut SynchronousDirectory> {
        self.files
            .get_mut(&fd)
            .and_then(|item| item.into_synchronous_directory())
    }

    pub fn pop_file_system_item(&mut self, fd: u32) -> Option<FileSystemItem> {
        self.files.remove(&fd)
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
    pub fn into_synchronous_file(&mut self) -> Option<&mut SynchronousFile> {
        match self {
            FileSystemItem::StandardInput(file) => Some(&mut file.file),
            FileSystemItem::StandardOutput(file) => Some(&mut file.file),
            FileSystemItem::StandardError(file) => Some(&mut file.file),
            FileSystemItem::File(file) => Some(&mut file.file),
            FileSystemItem::Directory(_) => None,
        }
    }

    pub fn into_synchronous_directory(&mut self) -> Option<&mut SynchronousDirectory> {
        match self {
            FileSystemItem::Directory(dir) => Some(&mut dir.directory),
            _ => None,
        }
    }
}
