#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[ignore]
#[xila::task::test(task_path = xila::task)]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate alloc;

    use xila::{
        executable::{self, mount_executables},
        task, virtual_file_system,
    };

    let standard = testing::initialize(false, true).await;

    let virtual_file_system = virtual_file_system::get_instance();
    let task = task::get_instance().get_current_task_identifier().await;

    mount_executables!(
        virtual_file_system,
        task,
        &[(
            &"/binaries/command_line_shell",
            command_line_shell::ShellExecutable
        ),]
    )
    .await
    .unwrap();

    let result = executable::execute("/binaries/command_line_shell", vec![], standard, None)
        .await
        .unwrap()
        .join()
        .await;

    assert!(result == 0);
}
