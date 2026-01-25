extern crate alloc;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[ignore]
#[xila::task::test(task_path = xila::task)]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate abi_definitions;

    use settings::SettingsExecutable;
    use xila::executable::mount_executables;
    use xila::{executable, task, virtual_file_system};

    let standard = testing::initialize(true, true).await;

    mount_executables!(
        virtual_file_system::get_instance(),
        task::get_instance().get_current_task_identifier().await,
        &[(&"/binaries/settings", SettingsExecutable),]
    )
    .await
    .unwrap();

    let result = executable::execute("/binaries/settings", vec![], standard, None)
        .await
        .unwrap()
        .join()
        .await;

    assert!(result == 0);
}
