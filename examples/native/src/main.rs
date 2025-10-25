#![no_std]

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
drivers_std::memory::instantiate_global_allocator!();

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[xila::task::run(task_path = xila::task, executor = drivers_std::executor::instantiate_static_executor!())]
async fn main() {
    extern crate alloc;

    use alloc::vec;
    use core::time::Duration;
    use xila::authentication;
    use xila::bootsplash::Bootsplash;
    use xila::executable;
    use xila::executable::Standard;
    use xila::executable::mount_static_executables;
    use xila::file_system;
    use xila::file_system::Mbr;
    use xila::file_system::PartitionKind;
    use xila::file_system::{create_device, create_file_system};
    use xila::graphics;
    use xila::host_bindings;
    use xila::little_fs;
    use xila::log;
    use xila::log::information;
    use xila::task;
    use xila::time;
    use xila::users;
    use xila::virtual_file_system;
    use xila::virtual_file_system::mount_static_devices;
    use xila::virtual_machine;

    // - Initialize the system
    log::initialize(&drivers_std::log::Logger).unwrap();

    // Initialize the task manager
    let task_manager = task::initialize();

    let task = task_manager.get_current_task_identifier().await;

    // Initialize the users manager
    users::initialize();
    // Initialize the time manager
    time::initialize(create_device!(drivers_native::TimeDriver::new())).unwrap();

    // - Initialize the graphics manager
    // - - Initialize the graphics driver
    const RESOLUTION: graphics::Point = graphics::Point::new(800, 600);
    let (screen_device, pointer_device, keyboard_device, mut event_loop) =
        drivers_native::window_screen::new(RESOLUTION)
            .await
            .unwrap();
    // - - Initialize the graphics manager
    let graphics_manager = graphics::initialize(
        screen_device,
        pointer_device,
        graphics::InputKind::Pointer,
        graphics::get_minimal_buffer_size(&RESOLUTION),
        true,
    )
    .await;

    graphics_manager
        .add_input_device(keyboard_device, graphics::InputKind::Keypad)
        .await
        .unwrap();

    let bootsplash = Bootsplash::new(graphics_manager).await.unwrap();

    task_manager
        .spawn(task, "Graphics", None, |_| {
            graphics_manager.r#loop(task::Manager::sleep)
        })
        .await
        .unwrap();

    task_manager
        .spawn(task, "Event Loop", None, async move |_| {
            event_loop.run().await
        })
        .await
        .unwrap();

    // - Initialize the file system
    // Create a memory device
    let drive = create_device!(drivers_std::drive_file::FileDriveDevice::new(
        &"./drive.img"
    ));

    // Create a partition type
    let partition = create_device!(
        Mbr::find_or_create_partition_with_signature(&drive, 0xDEADBEEF, PartitionKind::Xila)
            .unwrap()
    );

    // Print MBR information
    let mbr = Mbr::read_from_device(&drive).unwrap();

    information!("MBR information: {mbr}");

    // Mount the file system
    let file_system = match little_fs::FileSystem::new(partition.clone(), 256) {
        Ok(file_system) => file_system,
        // If the file system is not found, format it
        Err(_) => {
            partition
                .set_position(&file_system::Position::Start(0))
                .unwrap();

            little_fs::FileSystem::format(partition.clone(), 256).unwrap();

            little_fs::FileSystem::new(partition, 256).unwrap()
        }
    };
    // Initialize the virtual file system
    virtual_file_system::initialize(create_file_system!(file_system), None).unwrap();

    // - - Mount the devices

    // - - Create the default system hierarchy
    let _ =
        virtual_file_system::create_default_hierarchy(virtual_file_system::get_instance(), task)
            .await;

    // - - Mount the devices
    virtual_file_system::clean_devices(virtual_file_system::get_instance())
        .await
        .unwrap();

    mount_static_devices!(
        virtual_file_system::get_instance(),
        task,
        &[
            (
                &"/devices/standard_in",
                drivers_std::console::StandardInDevice
            ),
            (
                &"/devices/standard_out",
                drivers_std::console::StandardOutDevice
            ),
            (
                &"/devices/standard_error",
                drivers_std::console::StandardErrorDevice
            ),
            (&"/devices/time", drivers_native::TimeDriver),
            (&"/devices/random", drivers_shared::devices::RandomDevice),
            (&"/devices/null", drivers_core::NullDevice)
        ]
    )
    .await
    .unwrap();

    // Initialize the virtual machine
    virtual_machine::initialize(&[&host_bindings::GraphicsBindings]);

    // Mount static executables

    let virtual_file_system = virtual_file_system::get_instance();

    mount_static_executables!(
        virtual_file_system,
        task,
        &[
            (
                &"/binaries/graphical_shell",
                graphical_shell::ShellExecutable
            ),
            (
                &"/binaries/file_manager",
                file_manager::FileManagerExecutable::new(virtual_file_system, task)
                    .await
                    .unwrap()
            ),
            (
                &"/binaries/command_line_shell",
                command_line_shell::ShellExecutable
            ),
            (
                &"/binaries/terminal",
                terminal::TerminalExecutable::new(virtual_file_system::get_instance(), task)
                    .await
                    .unwrap()
            ),
            (
                &"/binaries/settings",
                settings::SettingsExecutable::new(virtual_file_system, task)
                    .await
                    .unwrap()
            ),
            (&"/binaries/wasm", wasm::WasmDevice)
        ]
    )
    .await
    .unwrap();

    // - Execute the shell
    // - - Open the standard input, output and error
    let standard = Standard::open(
        &"/devices/standard_in",
        &"/devices/standard_out",
        &"/devices/standard_error",
        task,
        virtual_file_system::get_instance(),
    )
    .await
    .unwrap();

    // - - Create the default user
    let group_identifier = users::GroupIdentifier::new(1000);

    let _ = authentication::create_group(
        virtual_file_system::get_instance(),
        "administrator",
        Some(group_identifier),
    )
    .await;

    let _ = authentication::create_user(
        virtual_file_system::get_instance(),
        "administrator",
        "",
        group_identifier,
        None,
    )
    .await;

    // - - Set the environment variables
    task_manager
        .set_environment_variable(task, "Paths", "/")
        .await
        .unwrap();

    task_manager
        .set_environment_variable(task, "Host", "xila")
        .await
        .unwrap();

    // wait some time to show the bootsplash
    task::Manager::sleep(Duration::from_secs(2)).await;

    bootsplash.stop(graphics_manager).await.unwrap();

    // - - Execute the shell
    let _ = executable::execute("/binaries/graphical_shell", vec![], standard)
        .await
        .unwrap()
        .join()
        .await;

    virtual_file_system::get_instance().uninitialize().await;
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn main() {
    panic!("This example is only for native platforms.");
}
