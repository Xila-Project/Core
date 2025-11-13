use exported_file_system::{
    BaseOperations, DirectoryOperations, FileIdentifier, FileSystemOperations, Mode, Permission,
    Permissions, Time, UniqueFileIdentifier,
};
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use task::TaskIdentifier;
use users::{GroupIdentifier, UserIdentifier};

use crate::{
    Error, Result, VirtualFileSystem,
    file_system::{ContextsMap, InternalContext, Kind, Path},
};

impl<'a> VirtualFileSystem<'a> {
    pub async fn perform_file_operation<R>(
        &self,
        file: UniqueFileIdentifier,
        mode: Mode,
        operation: impl FnOnce(&dyn BaseOperations, &mut InternalContext) -> Result<R>,
    ) -> Result<R> {
        let contexts = self.contexts.read().await;

        let mut context = contexts
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?
            .write()
            .await;

        if !context.flags.get_mode().allows(mode) {
            return Err(Error::PermissionDenied);
        }

        let file_operation = context
            .item
            .as_file_operations()
            .ok_or(Error::UnsupportedOperation)?;

        operation(file_operation, &mut context)
    }

    pub async fn perform_directory_operation<R>(
        &self,
        file: UniqueFileIdentifier,
        mode: Mode,
        operation: impl FnOnce(&dyn DirectoryOperations, &mut InternalContext) -> Result<R>,
    ) -> Result<R> {
        let contexts = self.contexts.read().await;

        let mut context = contexts
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?
            .write()
            .await;

        let directory_operations = context
            .item
            .as_directory_operations()
            .ok_or(Error::UnsupportedOperation)?;

        if !context.flags.get_mode().allows(mode) {
            return Err(Error::PermissionDenied);
        }

        operation(directory_operations, &mut context)
    }

    pub async fn add_context(
        &self,
        task_identifier: TaskIdentifier,
        file_identifier: Option<FileIdentifier>,
        context: RwLock<CriticalSectionRawMutex, InternalContext>,
    ) -> Result<UniqueFileIdentifier> {
        let mut contexts = self.contexts.write().await;

        let unique_file_identifier = match file_identifier {
            Some(file_identifier) => UniqueFileIdentifier::new(task_identifier, file_identifier),
            None => Self::get_new_file_identifier(&contexts, task_identifier)
                .ok_or(Error::TooManyOpenFiles)?,
        };

        contexts.insert(unique_file_identifier, context);

        Ok(unique_file_identifier)
    }

    pub async fn close_internal(
        contexts: &mut ContextsMap,
        file: UniqueFileIdentifier,
    ) -> Result<()> {
        let context = contexts
            .remove(&file)
            .ok_or(crate::Error::InvalidIdentifier)?;

        let mut context = context.write().await;

        if let Some(directory) = context.item.as_directory_operations() {
            directory.close(&mut context.context)?;
        } else if let Some(file_system) = context.item.as_file_operations() {
            file_system.close(&mut context.context)?;
        }

        Ok(())
    }

    pub async fn has_permissions(
        users_manager: &users::Manager,
        current_user: UserIdentifier,
        necessary_permissions: Permission,
        owner_user: UserIdentifier,
        owner_group: GroupIdentifier,
        permissions: Permissions,
    ) -> bool {
        if current_user == owner_user {
            if permissions.get_user().include(necessary_permissions) {
                return true;
            }
        }

        if users_manager.is_in_group(current_user, owner_group).await {
            if permissions.get_group().include(necessary_permissions) {
                return true;
            }
        }

        if permissions.get_others().include(necessary_permissions) {
            return true;
        }

        false
    }

    pub fn get_time(&self) -> Result<Time> {
        Ok(time::get_instance().get_current_time()?.into())
    }

    pub async fn get_time_user_group(
        &self,
        task: TaskIdentifier,
    ) -> Result<(Time, UserIdentifier, GroupIdentifier)> {
        let time: Time = time::get_instance().get_current_time()?.into();

        let user = task::get_instance().get_user(task).await?;

        let group = users::get_instance().get_user_primary_group(user).await?;

        Ok((time, user, group))
    }
}
