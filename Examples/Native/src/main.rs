#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

use xila::authentication;
use xila::drivers;
use xila::drivers::standard_library::executor::Instantiate_static_executor;
use xila::executable;
use xila::executable::Mount_static_executables;
use xila::executable::Standard_type;
use xila::file_system;
use xila::file_system::MBR_type;
use xila::file_system::Partition_type_type;
use xila::file_system::{Create_file_system, create_device};
use xila::graphics;
use xila::host_bindings;
use xila::little_fs;
use xila::log;
use xila::log::Information;
use xila::memory::Instantiate_global_allocator;
use xila::task;
use xila::time;
use xila::users;
use xila::virtual_file_system;
use xila::virtual_file_system::Mount_static_devices;
use xila::virtual_machine;

use alloc::string::String;

Instantiate_global_allocator!(drivers::standard_library::memory::Memory_manager_type);

#[task::Run(task_path = task, executor = Instantiate_static_executor!())]
async fn main() {
    // - Initialize the system
    log::initialize(&drivers::standard_library::log::Logger_type).unwrap();

    // Initialize the task manager
    let task_manager = task::initialize();

    let task = task_manager.get_current_task_identifier().await;

    // Initialize the users manager
    users::initialize();
    // Initialize the time manager
    time::initialize(create_device!(drivers::native::Time_driver_type::new())).unwrap();

    // - Initialize the graphics manager
    // - - Initialize the graphics driver
    const RESOLUTION: graphics::Point_type = graphics::Point_type::new(800, 600);
    let (screen_device, pointer_device, keyboard_device) =
        drivers::native::window_screen::new(RESOLUTION).unwrap();
    // - - Initialize the graphics manager
    let graphics_manager = graphics::initialize(
        screen_device,
        pointer_device,
        graphics::Input_type_type::Pointer,
        graphics::get_minimal_buffer_size(&RESOLUTION),
        true,
    )
    .await;

    graphics_manager
        .add_input_device(keyboard_device, graphics::Input_type_type::Keypad)
        .await
        .unwrap();

    task_manager
        .spawn(task, "Graphics", None, |_| {
            graphics_manager.r#loop(task::Manager_type::sleep)
        })
        .await
        .unwrap();

    // - Initialize the file system
    // Create a memory device
    let drive = create_device!(
        drivers::standard_library::drive_file::File_drive_device_type::new(&"./Drive.img")
    );

    // Create a partition type
    let partition = create_device!(
        MBR_type::find_or_create_partition_with_signature(
            &drive,
            0xDEADBEEF,
            Partition_type_type::Xila
        )
        .unwrap()
    );

    // Print MBR information
    let mbr = MBR_type::read_from_device(&drive).unwrap();

    Information!("MBR information: {mbr}");

    // Mount the file system
    let file_system = match little_fs::File_system_type::new(partition.clone(), 256) {
        Ok(file_system) => file_system,
        // If the file system is not found, format it
        Err(_) => {
            partition
                .set_position(&file_system::Position_type::Start(0))
                .unwrap();

            little_fs::File_system_type::format(partition.clone(), 256).unwrap();

            little_fs::File_system_type::new(partition, 256).unwrap()
        }
    };
    // Initialize the virtual file system
    virtual_file_system::initialize(Create_file_system!(file_system), None).unwrap();

    // - - Mount the devices

    // - - Create the default system hierarchy
    let _ =
        virtual_file_system::create_default_hierarchy(virtual_file_system::get_instance(), task)
            .await;

    // - - Mount the devices
    virtual_file_system::clean_devices(virtual_file_system::get_instance())
        .await
        .unwrap();

    Mount_static_devices!(
        virtual_file_system::get_instance(),
        task,
        &[
            (
                &"/Devices/Standard_in",
                drivers::standard_library::console::Standard_in_device_type
            ),
            (
                &"/Devices/Standard_out",
                drivers::standard_library::console::Standard_out_device_type
            ),
            (
                &"/Devices/Standard_error",
                drivers::standard_library::console::Standard_error_device_type
            ),
            (&"/Devices/Time", drivers::native::Time_driver_type),
            (&"/Devices/Random", drivers::native::Random_device_type),
            (&"/Devices/Null", drivers::core::Null_device_type)
        ]
    )
    .await
    .unwrap();

    // Initialize the virtual machine
    virtual_machine::initialize(&[&host_bindings::Graphics_bindings]);

    // Mount static executables

    let virtual_file_system = virtual_file_system::get_instance();

    Mount_static_executables!(
        virtual_file_system,
        task,
        &[
            (
                &"/Binaries/Graphical_shell",
                graphical_shell::Shell_executable_type
            ),
            (
                &"/Binaries/File_manager",
                file_manager::File_manager_executable_type::new(virtual_file_system, task)
                    .await
                    .unwrap()
            ),
            (
                &"/Binaries/Command_line_shell",
                command_line_shell::Shell_executable_type
            ),
            (
                &"/Binaries/Terminal",
                terminal::Terminal_executable_type::new(virtual_file_system::get_instance(), task)
                    .await
                    .unwrap()
            ),
            (
                &"/Binaries/Settings",
                settings::Settings_executable_type::new(virtual_file_system, task)
                    .await
                    .unwrap()
            ),
            (&"/Binaries/WASM", wasm::WASM_device_type)
        ]
    )
    .await
    .unwrap();

    // - Execute the shell
    // - - Open the standard input, output and error
    let standard = Standard_type::open(
        &"/Devices/Standard_in",
        &"/Devices/Standard_out",
        &"/Devices/Standard_error",
        task,
        virtual_file_system::get_instance(),
    )
    .await
    .unwrap();

    // - - Create the default user
    let group_identifier = users::Group_identifier_type::new(1000);

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
    let _ = executable::execute("/Binaries/Graphical_shell", String::from(""), standard)
        .await
        .unwrap()
        .join()
        .await;

    virtual_file_system::get_instance().uninitialize().await;
}
