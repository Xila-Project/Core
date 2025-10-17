#![no_std]

#[cfg(target_arch = "wasm32")]
xila::drivers::wasm::memory::instantiate_global_allocator!();

#[cfg(target_arch = "wasm32")]
#[xila::task::run(task_path = xila::task, executor = xila::drivers::wasm::executor::instantiate_static_executor!())]
async fn main() {
    extern crate alloc;

    use alloc::string::String;
    use xila::drivers::wasm::devices::graphics::GraphicsDevices;
    use xila::executable::{self, mount_static_executables, Standard};
    use xila::file_system::{self, create_device, create_file_system, Mbr, PartitionKind};
    use xila::log;
    use xila::task;
    use xila::time;
    use xila::virtual_file_system::mount_static_devices;
    use xila::{authentication, drivers, graphics, little_fs, users, virtual_file_system};

    console_error_panic_hook::set_once();

    // - Initialize the system
    log::initialize(&drivers::wasm::log::Logger).unwrap();

    // Initialize the task manager
    let task_manager = task::initialize();

    let task = task_manager.get_current_task_identifier().await;

    // Initialize the users manager
    users::initialize();
    // Initialize the time manager
    let _ = time::initialize(create_device!(drivers::wasm::devices::TimeDevice::new())).unwrap();

    // - Initialize the graphics manager
    // - - Initialize the graphics driver
    let GraphicsDevices {
        screen_device,
        mouse_device,
        keyboard_device,
        canvas: _canvas,
    } = drivers::wasm::devices::graphics::new().await.unwrap();

    let resolution = drivers::wasm::devices::graphics::get_resolution().unwrap();

    // - - Initialize the graphics manager
    let graphics_manager = graphics::initialize(
        screen_device,
        mouse_device,
        graphics::InputKind::Pointer,
        graphics::get_recommended_buffer_size(&resolution),
        //graphics::get_minimal_buffer_size(&resolution),
        true,
    )
    .await;

    graphics_manager
        .add_input_device(keyboard_device, graphics::InputKind::Keypad)
        .await
        .unwrap();

    task_manager
        .spawn(task, "Graphics", None, move |_| {
            graphics_manager.r#loop(task::Manager::sleep)
        })
        .await
        .unwrap();

    // - Initialize the file system
    // Create a memory device
    let drive = create_device!(file_system::MemoryDevice::<512>::new(16 * 1024 * 1024));
    //let drive = create_device!(drivers::wasm::devices::DriveDevice::new(Path::new("xila_drive.img")));

    // Create a partition type
    let partition = create_device!(Mbr::find_or_create_partition_with_signature(
        &drive,
        0xDEADBEEF,
        PartitionKind::Xila
    )
    .unwrap());

    // Print MBR information
    let mbr = Mbr::read_from_device(&drive).unwrap();

    log::information!("MBR Information: {mbr}");

    // Mount the file system
    let file_system = match little_fs::FileSystem::new(partition.clone(), 512) {
        Ok(file_system) => file_system,
        // If the file system is not found, format it
        Err(_) => {
            partition
                .set_position(&file_system::Position::Start(0))
                .unwrap();

            little_fs::FileSystem::format(partition.clone(), 512).unwrap();

            little_fs::FileSystem::new(partition, 512).unwrap()
        }
    };
    // Initialize the virtual file system
    let virtual_file_system =
        virtual_file_system::initialize(create_file_system!(file_system), None).unwrap();

    // - - Mount the devices

    // - - Create the default system hierarchy
    let _ = virtual_file_system::create_default_hierarchy(virtual_file_system, task).await;

    // - - Mount the devices
    virtual_file_system::clean_devices(virtual_file_system)
        .await
        .unwrap();

    mount_static_devices!(
        virtual_file_system,
        task,
        &[
            (&"/devices/standard_in", drivers::core::NullDevice),
            (&"/devices/standard_out", drivers::core::NullDevice),
            (&"/devices/standard_error", drivers::core::NullDevice),
            (&"/devices/time", drivers::wasm::devices::TimeDevice),
            (&"/devices/random", drivers::shared::devices::RandomDevice),
            (&"/devices/null", drivers::core::NullDevice)
        ]
    )
    .await
    .unwrap();

    // Mount static executables

    let virtual_file_system = virtual_file_system;

    mount_static_executables!(
        virtual_file_system,
        task,
        &[
            (
                &"/binaries/graphical_shell",
                graphical_shell::ShellExecutable
            ),
            (
                &"/binaries/command_line_shell",
                command_line_shell::ShellExecutable
            ),
            (
                &"/binaries/settings",
                settings::SettingsExecutable::new(virtual_file_system, task)
                    .await
                    .unwrap()
            ),
            (
                &"/binaries/file_manager",
                file_manager::FileManagerExecutable::new(virtual_file_system, task)
                    .await
                    .unwrap()
            ),
            (
                &"/binaries/terminal",
                terminal::TerminalExecutable::new(virtual_file_system, task)
                    .await
                    .unwrap()
            ),
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
        virtual_file_system,
    )
    .await
    .unwrap();

    // - - Create the default user
    let group_identifier = users::GroupIdentifier::new(1000);

    let _ =
        authentication::create_group(virtual_file_system, "administrator", Some(group_identifier))
            .await
            .unwrap();

    let _ = authentication::create_user(
        virtual_file_system,
        "administrator",
        "",
        group_identifier,
        None,
    )
    .await
    .unwrap();

    // - - Set the environment variables
    task_manager
        .set_environment_variable(task, "Paths", "/")
        .await
        .unwrap();

    task_manager
        .set_environment_variable(task, "Host", "xila")
        .await
        .unwrap();

    // - - Execute the shell
    let _ = executable::execute("/binaries/graphical_shell", String::from(""), standard)
        .await
        .unwrap()
        .join()
        .await;

    virtual_file_system.uninitialize().await;
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    extern crate std;

    panic!("This executable is only for the WebAssembly target.");
}
