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
    use file_manager::FileManagerExecutable;
    use xila::executable::Standard;
    use xila::executable::initialize_for_tests;
    use xila::executable::mount_executables;
    use xila::graphics::Point;
    use xila::virtual_file_system::mount_static;
    use xila::{executable, task, virtual_file_system};

    let (screen_device, pointer_device, keyboard_device, mut runner) =
        window_screen::new(Point::new(800, 600)).await.unwrap();

    initialize_for_tests(
        &Logger,
        &TimeDevice,
        &RandomDevice,
        Some((screen_device, pointer_device)),
        Some(keyboard_device),
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
        &[(&"/binaries/file_manager", FileManagerExecutable),]
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

    let standard = Standard::open(
        &"/devices/standard_in",
        &"/devices/standard_out",
        &"/devices/standard_error",
        task,
        virtual_file_system,
    )
    .await
    .unwrap();

    let result = executable::execute("/binaries/file_manager", vec![], standard, None)
        .await
        .unwrap()
        .join()
        .await;

    assert!(result == 0);
}
