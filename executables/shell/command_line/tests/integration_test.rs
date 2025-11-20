#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[ignore]
#[xila::task::test(task_path = xila::task)]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate alloc;

    use command_line_shell::ShellExecutable;
    use drivers_native::TimeDevice;
    use drivers_shared::devices::RandomDevice;
    use drivers_std::log::Logger;
    use xila::executable::{self, Standard, initialize_for_tests, mount_executables};
    use xila::virtual_file_system::mount_static;
    use xila::{task, virtual_file_system};

    initialize_for_tests(&Logger, &TimeDevice, &RandomDevice, None, None).await;

    let virtual_file_system = virtual_file_system::get_instance();
    let task = task::get_instance().get_current_task_identifier().await;

    mount_static!(
        virtual_file_system,
        task,
        &[
            (
                "/devices/standard_in",
                CharacterDevice,
                drivers_std::console::StandardInDevice
            ),
            (
                "/devices/standard_out",
                CharacterDevice,
                drivers_std::console::StandardOutDevice
            ),
            (
                "/devices/standard_error",
                CharacterDevice,
                drivers_std::console::StandardErrorDevice
            ),
            ("/devices/time", CharacterDevice, drivers_native::TimeDevice),
            (
                "/devices/random",
                CharacterDevice,
                drivers_shared::devices::RandomDevice
            ),
            ("/devices/null", CharacterDevice, drivers_core::NullDevice)
        ]
    )
    .await
    .unwrap();

    mount_executables!(
        virtual_file_system,
        task,
        &[(&"/binaries/command_line_shell", ShellExecutable)]
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

    let result = executable::execute("/binaries/command_line_shell", vec![], standard, None)
        .await
        .unwrap()
        .join()
        .await;

    assert!(result == 0);
}
