extern crate alloc;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[ignore]
#[xila::task::test(task_path = xila::task)]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate abi_definitions;

    use command_line_shell::ShellExecutable;
    use terminal::TerminalExecutable;
    use xila::executable::mount_executables;
    use xila::{executable, task, virtual_file_system};

    let standard = testing::initialize(true).await;

    let virtual_file_system = virtual_file_system::get_instance();
    let task_instance = task::get_instance();
    let task = task_instance.get_current_task_identifier().await;

    mount_executables!(
        virtual_file_system,
        task,
        &[
            (
                &"/binaries/terminal",
                TerminalExecutable::new(virtual_file_system, task)
                    .await
                    .unwrap()
            ),
            (&"/binaries/command_line_shell", ShellExecutable),
        ]
    )
    .await
    .unwrap();

    let result = executable::execute("/binaries/terminal", vec![], standard, None)
        .await
        .unwrap()
        .join()
        .await;

    assert!(result == 0);
}
