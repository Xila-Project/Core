extern crate alloc;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[ignore]
#[xila::task::test(task_path = xila::task)]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate abi_definitions;

    use graphical_shell::ShellExecutable;
    use xila::executable::mount_executables;
    use xila::virtual_file_system::File;
    use xila::{executable, task, virtual_file_system};

    let standard = testing::initialize(true).await;

    let task_manager = task::get_instance();
    let virtual_file_system = virtual_file_system::get_instance();
    let task = task_manager.get_current_task_identifier().await;

    mount_executables!(
        virtual_file_system,
        task,
        &[(&"/binaries/graphical_shell", ShellExecutable),]
    )
    .await
    .unwrap();

    virtual_file_system
        .create_directory(task, &"/configuration/shared/shortcuts")
        .await
        .unwrap();

    // Add fake shortcuts.
    for i in 0..20 {
        use alloc::format;

        File::write_to_path(
            virtual_file_system,
            task,
            format!("/configuration/shared/shortcuts/test{i}.json").as_str(),
            get_shortcut_string(i).as_bytes(),
        )
        .await
        .unwrap();
    }

    let result = executable::execute("/binaries/graphical_shell", vec![], standard, None)
        .await
        .unwrap()
        .join()
        .await;

    assert!(result == 0);
}

fn get_shortcut_string(index: usize) -> alloc::string::String {
    alloc::format!(
        r#"{{
    "name": "Test{index}",
    "command": "/binaries/?",
    "arguments": ["test"],
    "terminal": false,
    "icon_string": "T!",
    "icon_color": [255, 0, 0]
}}"#
    )
}
