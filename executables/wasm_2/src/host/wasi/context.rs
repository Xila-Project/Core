use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use xila::{
    file_system::PathOwned,
    task::TaskIdentifier,
    virtual_file_system::{SynchronousDirectory, SynchronousFile},
};

pub struct WasiContext {
    pub files: BTreeMap<i32, FileSystemItem>,
    pub args: Vec<Vec<u8>>,
    pub task: TaskIdentifier,
    pub random_state: u64,
    pub prestats: Vec<Prestat>,
    pub exit_code: Option<i32>,
}

pub struct Prestat {
    pub name: Vec<u8>,
}

pub enum FileVariantKind {
    Regular,
    StandardInput,
    StandardOutput,
    StandardError,
}

pub struct FileVariant {
    pub file: SynchronousFile,
    pub kind: FileVariantKind,
}

pub struct DirectoryVariant {
    pub path: PathOwned,
    pub directory: SynchronousDirectory,
}

pub enum FileSystemItem {
    File(FileVariant),
    Directory(DirectoryVariant),
}
