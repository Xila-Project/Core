mod context;
mod directory;
mod file;
mod file_system;
mod manager;
mod utilities;

use ::file_system::DirectCharacterDevice;
pub use context::*;
pub use directory::*;
pub use file::*;
pub use file_system::*;
pub use manager::*;
use synchronization::once_lock::OnceLock;

static MANAGER_INSTANCE: OnceLock<Manager> = OnceLock::new();

pub fn get_instance() -> &'static Manager<'static> {
    MANAGER_INSTANCE
        .try_get()
        .expect("Manager is not initialized")
}

pub fn initialize(
    _task_manager: &'static task::Manager,
    random_device: &'static dyn DirectCharacterDevice,
) -> &'static Manager<'static> {
    MANAGER_INSTANCE.get_or_init(|| Manager::new(random_device))
}
