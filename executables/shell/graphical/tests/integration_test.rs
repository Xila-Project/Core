extern crate alloc;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[ignore]
#[xila::task::test(task_path = xila::task)]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate abi_definitions;

    use drivers_native::TimeDevice;
    use drivers_native::window_screen;
    use drivers_shared::devices::RandomDevice;
    use drivers_std::log::Logger;
    use graphical_shell::ShellExecutable;
    use xila::executable::Standard;
    use xila::executable::initialize_for_tests;
    use xila::executable::mount_executables;
    use xila::graphics::Point;
    use xila::virtual_file_system::File;
    use xila::virtual_file_system::mount_static;
    use xila::{executable, task, virtual_file_system};

    let (screen_device, pointer_device, keyboard_device, mut runner) =
        window_screen::new(Point::new(800, 600)).await.unwrap();

    initialize_for_tests(
        &Logger,
        &TimeDevice,
        &RandomDevice,
        Some((Box::new(screen_device), Box::new(pointer_device))),
        Some(Box::new(keyboard_device)),
    )
    .await;

    let task_manager = task::get_instance();
    let virtual_file_system = virtual_file_system::get_instance();
    let task = task_manager.get_current_task_identifier().await;

    task_manager
        .spawn(task, "Window screen runner", None, async move |_| {
            runner.run().await;
        })
        .await
        .unwrap();

    mount_executables!(
        virtual_file_system,
        task,
        &[(&"/binaries/graphical_shell", ShellExecutable),]
    )
    .await
    .unwrap();

    mount_static!(
        virtual_file_system,
        task,
        &[
            (
                &"/devices/standard_in",
                CharacterDevice,
                drivers_std::console::StandardInDevice
            ),
            (
                &"/devices/standard_out",
                CharacterDevice,
                drivers_std::console::StandardOutDevice
            ),
            (
                &"/devices/standard_error",
                CharacterDevice,
                drivers_std::console::StandardErrorDevice
            ),
            (
                &"/devices/time",
                CharacterDevice,
                drivers_native::TimeDevice
            ),
            (&"/devices/null", CharacterDevice, drivers_core::NullDevice)
        ]
    )
    .await
    .unwrap();

    virtual_file_system
        .create_directory(&"/configuration/shared/shortcuts", task)
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

    let standard = Standard::open(
        &"/devices/standard_in",
        &"/devices/standard_out",
        &"/devices/standard_error",
        task,
        virtual_file_system,
    )
    .await
    .unwrap();

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
