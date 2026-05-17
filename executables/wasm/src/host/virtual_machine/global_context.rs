use core::ptr::NonNull;

use wamr_rust_sdk::sys::{WASMExecEnv, WASMModuleCommon};
use xila::{
    synchronization::{
        blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, rwlock::RwLock,
    },
    task::{block_on, yield_now},
};

use crate::host::bindings::common::EnvironmentContext;

static GLOBAL_CONTEXT: OnceLock<RwLock<CriticalSectionRawMutex, Option<GlobalContext>>> =
    OnceLock::new();

pub fn get_synchronous() -> Option<GlobalContext> {
    block_on(get())
}

pub fn set_synchronous(global_context: GlobalContext) {
    block_on(set(global_context));
}

pub fn clear_synchronous() {
    block_on(clear());
}

pub async fn get() -> Option<GlobalContext> {
    let guard = GLOBAL_CONTEXT
        .get_or_init(|| RwLock::new(None))
        .read()
        .await;

    guard.clone()
}

pub async fn get_current_environment() -> Option<NonNull<WASMExecEnv>> {
    let global_context = get().await?;

    Some(global_context.current_environment)
}

pub async fn get_current_environment_context<'a>() -> Option<&'a mut EnvironmentContext> {
    let global_context = get().await?;

    EnvironmentContext::from_environment(global_context.current_environment)
}

pub fn get_current_environment_context_synchronous<'a>() -> Option<&'a mut EnvironmentContext> {
    block_on(get_current_environment_context())
}

pub async fn get_current_instance() -> Option<NonNull<WASMModuleCommon>> {
    let global_context = get().await?;

    Some(global_context.current_instance)
}

pub async fn set(global_context: GlobalContext) {
    loop {
        let mut guard = GLOBAL_CONTEXT
            .get_or_init(|| RwLock::new(None))
            .write()
            .await;

        if guard.is_none() {
            *guard = Some(global_context);
            return; // Success
        }

        drop(guard);

        yield_now().await;
    }
}

pub async fn clear() {
    let mut guard = GLOBAL_CONTEXT
        .get_or_init(|| RwLock::new(None))
        .write()
        .await;

    guard.take();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalContext {
    pub current_environment: NonNull<WASMExecEnv>,
    pub current_instance: NonNull<WASMModuleCommon>,
}

unsafe impl Send for GlobalContext {}
unsafe impl Sync for GlobalContext {}
