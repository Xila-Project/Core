#![no_std]

extern crate alloc;

mod file;
mod unique_file;

pub use file::*;

use alloc::{collections::btree_map::BTreeMap, vec, vec::Vec};
use file_system::{Path, PathOwned};
use futures::block_on;
use smol_str::SmolStr;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use task::TaskIdentifier;
use unique_file::UniqueFileIdentifier;
use virtual_file_system::{SynchronousDirectory, SynchronousFile};

pub static CONTEXT: Context = Context::new();

pub fn get_instance() -> &'static Context {
    &CONTEXT
}

struct DirectoryEntry {
    path: SmolStr,
    parent: Option<FileIdentifier>,
    directory: SynchronousDirectory,
}

type FileEntry = SynchronousFile;

struct Inner {
    task: Option<TaskIdentifier>,
    directories: BTreeMap<UniqueFileIdentifier, DirectoryEntry>,
    files: BTreeMap<UniqueFileIdentifier, FileEntry>,
}

pub struct Context(RwLock<CriticalSectionRawMutex, Inner>);

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub const fn new() -> Self {
        Self(RwLock::new(Inner {
            task: None,
            directories: BTreeMap::new(),
            files: BTreeMap::new(),
        }))
    }

    pub fn get_current_task_identifier(&self) -> TaskIdentifier {
        block_on(self.0.read()).task.expect("No current task set")
    }

    fn get_new_identifier<V>(
        map: &BTreeMap<UniqueFileIdentifier, V>,
        task: TaskIdentifier,
        start: FileIdentifier,
        end: FileIdentifier,
    ) -> Option<UniqueFileIdentifier> {
        let start_raw = start.into_inner();
        let end_raw = end.into_inner();

        // Find first available identifier by checking gaps in existing keys
        let mut current = start_raw;

        for key in map.keys() {
            let (key_task, key_file) = key.split();
            if key_task != task {
                continue;
            }

            let key_raw = key_file.into_inner();

            // Skip keys outside our range
            if key_raw < start_raw || key_raw > end_raw {
                continue;
            }

            // Found a gap before this key
            if current < key_raw {
                return FileIdentifier::new(current).map(|id| UniqueFileIdentifier::new(task, id));
            }

            // Move past this key
            current = key_raw.checked_add(1)?;
            if current > end_raw {
                break;
            }
        }

        // Check if there's space after all existing keys
        if current <= end_raw {
            return FileIdentifier::new(current).map(|id| UniqueFileIdentifier::new(task, id));
        }

        None
    }

    fn get_new_identifier_file(
        map: &BTreeMap<UniqueFileIdentifier, FileEntry>,
        task: TaskIdentifier,
    ) -> Option<UniqueFileIdentifier> {
        Self::get_new_identifier(
            map,
            task,
            FileIdentifier::MINIMUM_FILE,
            FileIdentifier::MAXIMUM_FILE,
        )
    }

    fn get_new_identifier_directory(
        map: &BTreeMap<UniqueFileIdentifier, DirectoryEntry>,
        task: TaskIdentifier,
    ) -> Option<UniqueFileIdentifier> {
        Self::get_new_identifier(
            map,
            task,
            FileIdentifier::MINIMUM_DIRECTORY,
            FileIdentifier::MAXIMUM_DIRECTORY,
        )
    }

    pub fn insert_file(
        &self,
        task: TaskIdentifier,
        file: SynchronousFile,
        custom_file_identifier: Option<FileIdentifier>,
    ) -> Option<FileIdentifier> {
        let mut inner = block_on(self.0.write());

        let file_identifier = if let Some(custom_file_identifier) = custom_file_identifier {
            let file_identifier = UniqueFileIdentifier::new(task, custom_file_identifier);
            if inner.files.contains_key(&file_identifier) {
                panic!("File identifier {:?} is already in use", file_identifier);
            }
            file_identifier
        } else {
            Self::get_new_identifier_file(&inner.files, task).unwrap()
        };

        inner.files.insert(file_identifier, file);

        Some(file_identifier.get_file())
    }

    pub fn perform_operation_on_file_or_directory<FF, FD, O>(
        &self,
        file_identifier: FileIdentifier,
        operation_file: FF,
        operation_directory: FD,
    ) -> Option<O>
    where
        FF: FnOnce(&mut SynchronousFile) -> O,
        FD: FnOnce(&mut SynchronousDirectory) -> O,
    {
        let task = self.get_current_task_identifier();
        let unique_file = UniqueFileIdentifier::new(task, file_identifier);

        let mut inner = block_on(self.0.write());

        if file_identifier.is_directory() {
            inner
                .directories
                .get_mut(&unique_file)
                .map(|entry| operation_directory(&mut entry.directory))
        } else {
            inner.files.get_mut(&unique_file).map(operation_file)
        }
    }

    pub fn perform_operation_on_file<F, O>(
        &self,
        file_identifier: FileIdentifier,
        operation: F,
    ) -> Option<O>
    where
        F: FnOnce(&mut SynchronousFile) -> O,
    {
        let task = self.get_current_task_identifier();
        let file = UniqueFileIdentifier::new(task, file_identifier);

        let mut inner = block_on(self.0.write());
        let file = inner.files.get_mut(&file)?;

        Some(operation(file))
    }

    pub fn perform_operation_on_directory<F, O>(
        &self,
        file: FileIdentifier,
        operation: F,
    ) -> Option<O>
    where
        F: FnOnce(&mut SynchronousDirectory) -> O,
    {
        let task = self.get_current_task_identifier();
        let file = UniqueFileIdentifier::new(task, file);

        let mut inner = block_on(self.0.write());
        inner
            .directories
            .get_mut(&file)
            .map(|entry| operation(&mut entry.directory))
    }

    pub fn insert_directory(
        &self,
        task: TaskIdentifier,
        parent: Option<FileIdentifier>,
        path: impl AsRef<Path>,
        directory: SynchronousDirectory,
    ) -> Option<FileIdentifier> {
        let mut inner = block_on(self.0.write());

        let file_identifier = Self::get_new_identifier_directory(&inner.directories, task).unwrap();

        inner.directories.insert(
            file_identifier,
            DirectoryEntry {
                path: SmolStr::new(path.as_ref()),
                parent,
                directory,
            },
        );

        Some(file_identifier.get_file())
    }

    pub fn remove_directory(&self, file: FileIdentifier) -> Option<SynchronousDirectory> {
        let task = self.get_current_task_identifier();
        let file = UniqueFileIdentifier::new(task, file);

        let mut inner = block_on(self.0.write());
        inner.directories.remove(&file).map(|entry| entry.directory)
    }

    pub fn remove_file(&self, file: FileIdentifier) -> Option<SynchronousFile> {
        let task = self.get_current_task_identifier();
        let file = UniqueFileIdentifier::new(task, file);

        let mut inner = block_on(self.0.write());
        inner.files.remove(&file)
    }

    pub fn resolve_path(
        &self,
        task: TaskIdentifier,
        directory: FileIdentifier,
        path: impl AsRef<Path>,
    ) -> Option<PathOwned> {
        let inner = block_on(self.0.read());

        let mut stack: Vec<&SmolStr> = vec![];

        let mut new_size = path.as_ref().get_length();

        let mut current_file_identifier = directory.into_unique(task);

        loop {
            let DirectoryEntry { path, parent, .. } =
                inner.directories.get(&current_file_identifier)?;

            new_size += path.len() + 1; // +1 for the separator

            if let Some(parent) = parent {
                stack.push(path);
                current_file_identifier = parent.into_unique(task);
            } else {
                break;
            }
        }

        let mut new_path = PathOwned::new_with_capacity(new_size);

        while let Some(path) = stack.pop() {
            new_path = new_path.join(Path::from_str(path))?;
        }

        let new_path = new_path.join(path)?;

        Some(new_path)
    }

    pub async fn set_task(&self, task: TaskIdentifier) {
        loop {
            let mut inner = self.0.write().await;

            if inner.task.is_none() {
                inner.task.replace(task);
                break;
            }
        }
    }

    pub async fn clear_task(&self) {
        let mut inner = self.0.write().await;
        inner.task.take();
    }

    pub async fn call_abi<F, Fut, R>(&self, function: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = R>,
    {
        let task = task::get_instance().get_current_task_identifier().await;
        self.set_task(task).await;
        let result = function().await;
        self.clear_task().await;
        result
    }
}

#[cfg(test)]
mod tests {
    use core::mem::forget;

    use file_system::{AccessFlags, DummyFileSystem};

    use super::*;

    #[test]
    fn test_context_new() {
        let context = Context::new();
        assert!(block_on(context.0.read()).task.is_none());
        assert!(block_on(context.0.read()).directories.is_empty());
    }

    #[test]
    fn test_get_instance() {
        let _ = get_instance();
    }

    fn new_dummy_directory() -> SynchronousDirectory {
        SynchronousDirectory::new(
            &DummyFileSystem,
            AccessFlags::READ_WRITE.into(),
            file_system::Context::new_empty(),
        )
    }

    fn initialize() -> (TaskIdentifier, Context) {
        let context = Context::new();
        let task = TaskIdentifier::new(1);
        (task, context)
    }

    fn clean_up(context: &Context) {
        let mut inner = block_on(context.0.write());

        let keys = inner.directories.keys().cloned().collect::<Vec<_>>();

        for key in keys {
            let directory = inner.directories.remove(&key).unwrap();
            forget(directory); // Do not call drop explicitly since they are invalid
        }
    }

    #[test]
    fn test_insert_and_remove_opened_file_identifier_path() {
        let (task, context) = initialize();
        let parent_id = FileIdentifier::new_panic(10);
        let path = Path::from_str("test.txt");

        let file_identifier = context
            .insert_directory(task, Some(parent_id), path, new_dummy_directory())
            .unwrap();

        let inner = block_on(context.0.read());

        let unique_file_identifier = UniqueFileIdentifier::new(task, file_identifier);

        assert!(inner.directories.contains_key(&unique_file_identifier));
        drop(inner);

        // Set task before calling remove_directory since it calls get_current_task_identifier
        block_on(context.set_task(task));
        forget(context.remove_directory(file_identifier));
        block_on(context.clear_task());

        let inner = block_on(context.0.read());
        assert!(!inner.directories.contains_key(&unique_file_identifier));
        drop(inner);

        clean_up(&context);
    }

    #[test]
    fn test_insert_with_none_parent() {
        let (task, context) = initialize();
        let path = Path::from_str("test.txt");
        let directory = new_dummy_directory();

        let file_identifier = context
            .insert_directory(task, None, path, directory)
            .unwrap();

        let inner = block_on(context.0.read());
        let unique_file_identifier = UniqueFileIdentifier::new(task, file_identifier);
        let DirectoryEntry { parent, .. } = inner.directories.get(&unique_file_identifier).unwrap();
        assert_eq!(*parent, None);
        drop(inner);

        clean_up(&context);
    }

    #[test]
    fn test_get_full_path_single_level() {
        let (task, context) = initialize();
        let path = Path::from_str("base");
        let directory = new_dummy_directory();

        let base_id = context
            .insert_directory(task, None, path, directory)
            .unwrap();

        let result = context.resolve_path(task, base_id, Path::from_str("file.txt"));
        // Since base has no parent (INVALID), it stops and only includes the provided path
        assert_eq!(result.unwrap().as_str(), "/file.txt");

        clean_up(&context);
    }

    #[test]
    fn test_get_full_path_nested() {
        let (task, context) = initialize();

        let root_id = context
            .insert_directory(task, None, "root", new_dummy_directory())
            .unwrap();
        let dir_id = context
            .insert_directory(task, Some(root_id), "dir", new_dummy_directory())
            .unwrap();
        let sub_dir_id = context
            .insert_directory(task, Some(dir_id), "subdir", new_dummy_directory())
            .unwrap();

        let path = context
            .resolve_path(task, sub_dir_id, Path::from_str("file.txt"))
            .unwrap();
        // The algorithm stops when reaching a directory with INVALID parent (root)
        // So it builds path from children directories only, not including root
        assert_eq!(path.as_str(), "/dir/subdir/file.txt");

        clean_up(&context);
    }

    #[test]
    fn test_get_full_path_nonexistent() {
        let (task, context) = initialize();
        let file_id = FileIdentifier::new_panic(999);

        let result = context.resolve_path(task, file_id, Path::from_str("file.txt"));
        assert_eq!(result, None);

        clean_up(&context);
    }

    #[test]
    fn test_remove_nonexistent_file() {
        let (task, context) = initialize();
        let file_id = FileIdentifier::new_panic(999);

        block_on(context.set_task(task));
        let result = context.remove_directory(file_id);
        block_on(context.clear_task());

        assert!(result.is_none());
        clean_up(&context);
    }

    #[test]
    fn test_multiple_tasks() {
        let (task1, context) = initialize();
        let task2 = TaskIdentifier::new(2);

        let file_id1 = context
            .insert_directory(
                task1,
                None,
                Path::from_str("task1.txt"),
                new_dummy_directory(),
            )
            .unwrap();
        let file_id2 = context
            .insert_directory(
                task2,
                None,
                Path::from_str("task2.txt"),
                new_dummy_directory(),
            )
            .unwrap();

        let inner = block_on(context.0.read());
        let unique_id1 = UniqueFileIdentifier::new(task1, file_id1);
        let unique_id2 = UniqueFileIdentifier::new(task2, file_id2);

        assert!(inner.directories.contains_key(&unique_id1));
        assert!(inner.directories.contains_key(&unique_id2));
        drop(inner);

        clean_up(&context);
    }
}
