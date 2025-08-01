#![no_std]

extern crate alloc;

use xila::authentication;
use xila::drivers;
use xila::drivers::standard_library::executor::instantiate_static_executor;
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
use xila::log::Information;
use xila::memory::instantiate_global_allocator;
use xila::task;
use xila::time;
use xila::users;
use xila::virtual_file_system;
use xila::virtual_file_system::mount_static_devices;
use xila::virtual_machine;

use alloc::string::String;

instantiate_global_allocator!(drivers::standard_library::memory::MemoryManager);

#[task::run(task_path = task, executor = instantiate_static_executor!())]
async fn main() {
    // - Initialize the system
    log::initialize(&drivers::standard_library::log::Logger).unwrap();

    // Initialize the task manager
    let task_manager = task::initialize();

    let task = task_manager.get_current_task_identifier().await;

    // Initialize the users manager
    users::initialize();
    // Initialize the time manager
    time::initialize(create_device!(drivers::native::TimeDriver::new())).unwrap();

    // - Initialize the graphics manager
    // - - Initialize the graphics driver
    const RESOLUTION: graphics::Point = graphics::Point::new(800, 600);
    let (screen_device, pointer_device, keyboard_device) =
        drivers::native::window_screen::new(RESOLUTION).unwrap();
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

    task_manager
        .spawn(task, "Graphics", None, |_| {
            graphics_manager.r#loop(task::Manager::sleep)
        })
        .await
        .unwrap();

    // - Initialize the file system
    // Create a memory device
    let drive = create_device!(drivers::standard_library::drive_file::FileDriveDevice::new(
        &"./Drive.img"
    ));

    // Create a partition type
    let partition = create_device!(
        Mbr::find_or_create_partition_with_signature(&drive, 0xDEADBEEF, PartitionKind::Xila)
            .unwrap()
    );

    // Print MBR information
    let mbr = Mbr::read_from_device(&drive).unwrap();

    Information!("MBR information: {mbr}");

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
                drivers::standard_library::console::StandardInDevice
            ),
            (
                &"/devices/standard_out",
                drivers::standard_library::console::StandardOutDevice
            ),
            (
                &"/devices/standard_error",
                drivers::standard_library::console::StandardErrorDevice
            ),
            (&"/devices/time", drivers::native::TimeDriver),
            (&"/devices/random", drivers::native::RandomDevice),
            (&"/devices/null", drivers::core::NullDevice)
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
        "alix_anneraud",
        Some(group_identifier),
    )
    .await;

    let _ = authentication::create_user(
        virtual_file_system::get_instance(),
        "alix_anneraud",
        "password",
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
    // - - Execute the shell
    let _ = executable::execute("/binaries/graphical_shell", String::from(""), standard)
        .await
        .unwrap()
        .join()
        .await;

    virtual_file_system::get_instance().uninitialize().await;
}
