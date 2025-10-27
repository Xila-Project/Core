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
