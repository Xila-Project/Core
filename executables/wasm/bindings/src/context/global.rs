use xila::{
    synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock},
    task::{self, block_on},
};

use crate::{EnvironmentContext, InstanceContext};

static GLOBAL_CONTEXT: RwLock<CriticalSectionRawMutex, Option<GlobalContext>> = RwLock::new(None);

pub struct GlobalContext {
    pub environment_context: *mut EnvironmentContext,
    pub instance_context: *mut InstanceContext,
}

unsafe impl Send for GlobalContext {}
unsafe impl Sync for GlobalContext {}

impl GlobalContext {
    /// # Safety
    /// This function is unsafe because it returns a mutable reference to the global environment context, which can lead to data races if accessed from multiple threads without proper synchronization. It is the caller's responsibility to ensure that the returned reference is not accessed concurrently from multiple threads.
    pub unsafe fn get_environment_context<'a>() -> Option<&'a mut EnvironmentContext> {
        let guard = block_on(GLOBAL_CONTEXT.read());

        unsafe { guard.as_ref()?.environment_context.as_mut() }
    }

    /// # Safety
    /// This function is unsafe because it returns a mutable reference to the global instance context, which can lead to data races if accessed from multiple threads without proper synchronization. It is the caller's responsibility to ensure that the returned reference is not accessed concurrently from multiple threads.
    pub unsafe fn get_instance_context<'a>() -> Option<&'a mut InstanceContext> {
        let guard = block_on(GLOBAL_CONTEXT.read());

        unsafe { guard.as_ref()?.instance_context.as_mut() }
    }

    pub async fn set(
        instance_context: *mut InstanceContext,
        environment_context: *mut EnvironmentContext,
    ) {
        loop {
            {
                let mut global_context = GLOBAL_CONTEXT.write().await;
                if global_context.is_none() {
                    *global_context = Some(GlobalContext {
                        environment_context,
                        instance_context,
                    });
                    break;
                }
            }

            task::yield_now().await;
        }
    }

    pub async fn clear() {
        let mut global_context = GLOBAL_CONTEXT.write().await;
        *global_context = None;
    }
}
