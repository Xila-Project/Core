#![no_std]

extern crate alloc;

use alloc::{collections::btree_map::BTreeMap, vec, vec::Vec};
use file_system::{Path, PathOwned, UniqueFileIdentifier};
use futures::block_on;
use smol_str::SmolStr;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use task::TaskIdentifier;

pub static CONTEXT: Context = Context::new();

pub fn get_instance() -> &'static Context {
    &CONTEXT
}

struct Inner {
    task: Option<TaskIdentifier>,
    opened_file_identifiers_paths:
        BTreeMap<(TaskIdentifier, UniqueFileIdentifier), (SmolStr, UniqueFileIdentifier)>,
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
            opened_file_identifiers_paths: BTreeMap::new(),
        }))
    }

    pub fn get_current_task_identifier(&self) -> TaskIdentifier {
        block_on(self.0.read()).task.expect("No current task set")
    }

    pub fn insert_opened_file_identifier_path(
        &self,
        task: TaskIdentifier,
        file_identifier: UniqueFileIdentifier,
        parent_file_identifier: Option<UniqueFileIdentifier>,
        path: impl AsRef<Path>,
    ) {
        let mut inner = block_on(self.0.write());

        let parent_file_identifier =
            parent_file_identifier.unwrap_or(UniqueFileIdentifier::INVALID_FILE_IDENTIFIER);

        inner.opened_file_identifiers_paths.insert(
            (task, file_identifier),
            (SmolStr::new(path.as_ref()), parent_file_identifier),
        );
    }

    pub fn remove_opened_file_identifier_path(
        &self,
        task: TaskIdentifier,
        file_identifier: UniqueFileIdentifier,
    ) {
        let mut inner = block_on(self.0.write());
        inner
            .opened_file_identifiers_paths
            .remove(&(task, file_identifier));
    }

    pub fn get_full_path(
        &self,
        task: TaskIdentifier,
        file_identifier: UniqueFileIdentifier,
        path: impl AsRef<Path>,
    ) -> Option<PathOwned> {
        let inner = block_on(self.0.read());

        let mut stack: Vec<&SmolStr> = vec![];

        let mut new_size = path.as_ref().get_length();

        let mut current_file_identifier = file_identifier;

        while let Some((path, parent_file_identifier)) = inner
            .opened_file_identifiers_paths
            .get(&(task, current_file_identifier))
        {
            new_size += path.len() + 1; // +1 for the separator

            if *parent_file_identifier == UniqueFileIdentifier::INVALID_FILE_IDENTIFIER {
                break;
            }

            stack.push(path);
            current_file_identifier = *parent_file_identifier;
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
        Fut: core::future::Future<Output = R>,
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
    use super::*;

    #[test]
    fn test_context_new() {
        let context = Context::new();
        assert!(block_on(context.0.read()).task.is_none());
        assert!(
            block_on(context.0.read())
                .opened_file_identifiers_paths
                .is_empty()
        );
    }

    #[test]
    fn test_get_instance() {
        let _ = get_instance();
    }

    #[test]
    fn test_insert_and_remove_opened_file_identifier_path() {
        let context = Context::new();
        let task = TaskIdentifier::new(1);
        let file_id = UniqueFileIdentifier::from_raw(42);
        let parent_id = Some(UniqueFileIdentifier::from_raw(10));
        let path = Path::from_str("test.txt");

        context.insert_opened_file_identifier_path(task, file_id, parent_id, path);

        let inner = block_on(context.0.read());
        assert!(
            inner
                .opened_file_identifiers_paths
                .contains_key(&(task, file_id))
        );
        drop(inner);

        context.remove_opened_file_identifier_path(task, file_id);

        let inner = block_on(context.0.read());
        assert!(
            !inner
                .opened_file_identifiers_paths
                .contains_key(&(task, file_id))
        );
    }

    #[test]
    fn test_insert_with_none_parent() {
        let context = Context::new();
        let task = TaskIdentifier::new(1);
        let file_id = UniqueFileIdentifier::from_raw(42);
        let path = Path::from_str("test.txt");

        context.insert_opened_file_identifier_path(task, file_id, None, path);

        let inner = block_on(context.0.read());
        let (_, parent) = inner
            .opened_file_identifiers_paths
            .get(&(task, file_id))
            .unwrap();
        assert_eq!(*parent, UniqueFileIdentifier::INVALID_FILE_IDENTIFIER);
    }

    #[test]
    fn test_get_full_path_single_level() {
        let context = Context::new();
        let task = TaskIdentifier::new(1);
        let file_id = UniqueFileIdentifier::from_raw(42);
        let path = Path::from_str("base");

        context.insert_opened_file_identifier_path(task, file_id, None, path);

        let result = context.get_full_path(task, file_id, Path::from_str("file.txt"));
        assert!(result.is_some());
    }

    #[test]
    fn test_get_full_path_nested() {
        let context = Context::new();
        let task = TaskIdentifier::new(1);
        let root_id = UniqueFileIdentifier::from_raw(1);
        let dir_id = UniqueFileIdentifier::from_raw(2);
        let file_id = UniqueFileIdentifier::from_raw(3);

        context.insert_opened_file_identifier_path(task, root_id, None, Path::from_str("root"));
        context.insert_opened_file_identifier_path(
            task,
            dir_id,
            Some(root_id),
            Path::from_str("dir"),
        );
        context.insert_opened_file_identifier_path(
            task,
            file_id,
            Some(dir_id),
            Path::from_str("subdir"),
        );

        let result = context.get_full_path(task, file_id, Path::from_str("file.txt"));
        assert!(result.is_some());
    }

    #[test]
    fn test_get_full_path_nonexistent() {
        let context = Context::new();
        let task = TaskIdentifier::new(1);
        let file_id = UniqueFileIdentifier::from_raw(999);

        let result = context.get_full_path(task, file_id, Path::from_str("file.txt"));
        assert!(result.is_some());
    }

    #[test]
    fn test_remove_nonexistent_file() {
        let context = Context::new();
        let task = TaskIdentifier::new(1);
        let file_id = UniqueFileIdentifier::from_raw(999);

        context.remove_opened_file_identifier_path(task, file_id);

        let inner = block_on(context.0.read());
        assert!(
            !inner
                .opened_file_identifiers_paths
                .contains_key(&(task, file_id))
        );
    }

    #[test]
    fn test_multiple_tasks() {
        let context = Context::new();
        let task1 = TaskIdentifier::new(1);
        let task2 = TaskIdentifier::new(2);
        let file_id = UniqueFileIdentifier::from_raw(42);

        context.insert_opened_file_identifier_path(
            task1,
            file_id,
            None,
            Path::from_str("task1.txt"),
        );
        context.insert_opened_file_identifier_path(
            task2,
            file_id,
            None,
            Path::from_str("task2.txt"),
        );

        let inner = block_on(context.0.read());
        assert!(
            inner
                .opened_file_identifiers_paths
                .contains_key(&(task1, file_id))
        );
        assert!(
            inner
                .opened_file_identifiers_paths
                .contains_key(&(task2, file_id))
        );
    }
}
