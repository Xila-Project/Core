#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

use xila::Authentication;
use xila::Drivers;
use xila::Drivers::Std::Executor::Instantiate_static_executor;
use xila::Executable;
use xila::Executable::Mount_static_executables;
use xila::Executable::Standard_type;
use xila::File_system;
use xila::File_system::MBR_type;
use xila::File_system::Partition_type_type;
use xila::File_system::{Create_device, Create_file_system};
use xila::Graphics;
use xila::Host_bindings;
use xila::LittleFS;
use xila::Log;
use xila::Log::Information;
use xila::Memory::Instantiate_global_allocator;
use xila::Task;
use xila::Time;
use xila::Users;
use xila::Virtual_file_system;
use xila::Virtual_file_system::Mount_static_devices;
use xila::Virtual_machine;

use alloc::string::String;

Instantiate_global_allocator!(Drivers::Std::Memory::Memory_manager_type);

#[Task::Run(task_path = Task, executor = Instantiate_static_executor!())]
async fn main() {
    // - Initialize the system
    Log::Initialize(&Drivers::Std::Log::Logger_type).unwrap();

    // Initialize the task manager
    let task_manager = Task::Initialize();

    let task = task_manager.get_current_task_identifier().await;

    // Initialize the users manager
    Users::Initialize();
    // Initialize the time manager
    Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::new())).unwrap();

    // - Initialize the graphics manager
    // - - Initialize the graphics driver
    const RESOLUTION: Graphics::Point_type = Graphics::Point_type::new(800, 600);
    let (screen_device, pointer_device, keyboard_device) =
        Drivers::Native::Window_screen::New(RESOLUTION).unwrap();
    // - - Initialize the graphics manager
    let graphics_manager = Graphics::initialize(
        screen_device,
        pointer_device,
        Graphics::Input_type_type::Pointer,
        Graphics::get_minimal_buffer_size(&RESOLUTION),
        true,
    )
    .await;

    graphics_manager
        .add_input_device(keyboard_device, Graphics::Input_type_type::Keypad)
        .await
        .unwrap();

    task_manager
        .Spawn(task, "Graphics", None, |_| {
            graphics_manager.r#loop(Task::Manager_type::Sleep)
        })
        .await
        .unwrap();

    // - Initialize the file system
    // Create a memory device
    let drive = Create_device!(Drivers::Std::Drive_file::File_drive_device_type::new(
        &"./Drive.img"
    ));

    // Create a partition type
    let partition = Create_device!(
        MBR_type::Find_or_create_partition_with_signature(
            &drive,
            0xDEADBEEF,
            Partition_type_type::Xila
        )
        .unwrap()
    );

    // Print MBR information
    let mbr = MBR_type::Read_from_device(&drive).unwrap();

    Information!("MBR information: {mbr}");

    // Mount the file system
    let file_system = match LittleFS::File_system_type::new(partition.clone(), 256) {
        Ok(file_system) => file_system,
        // If the file system is not found, format it
        Err(_) => {
            partition
                .Set_position(&File_system::Position_type::Start(0))
                .unwrap();

            LittleFS::File_system_type::format(partition.clone(), 256).unwrap();

            LittleFS::File_system_type::new(partition, 256).unwrap()
        }
    };
    // Initialize the virtual file system
    Virtual_file_system::initialize(Create_file_system!(file_system), None).unwrap();

    // - - Mount the devices

    // - - Create the default system hierarchy
    let _ =
        Virtual_file_system::create_default_hierarchy(Virtual_file_system::get_instance(), task)
            .await;

    // - - Mount the devices
    Virtual_file_system::clean_devices(Virtual_file_system::get_instance())
        .await
        .unwrap();

    Mount_static_devices!(
        Virtual_file_system::get_instance(),
        task,
        &[
            (
                &"/Devices/Standard_in",
                Drivers::Std::Console::Standard_in_device_type
            ),
            (
                &"/Devices/Standard_out",
                Drivers::Std::Console::Standard_out_device_type
            ),
            (
                &"/Devices/Standard_error",
                Drivers::Std::Console::Standard_error_device_type
            ),
            (&"/Devices/Time", Drivers::Native::Time_driver_type),
            (&"/Devices/Random", Drivers::Native::Random_device_type),
            (&"/Devices/Null", Drivers::Core::Null_device_type)
        ]
    )
    .await
    .unwrap();

    // Initialize the virtual machine
    Virtual_machine::Initialize(&[&Host_bindings::Graphics_bindings]);

    // Mount static executables

    let virtual_file_system = Virtual_file_system::get_instance();

    Mount_static_executables!(
        virtual_file_system,
        task,
        &[
            (
                &"/Binaries/Graphical_shell",
                Graphical_shell::Shell_executable_type
            ),
            (
                &"/Binaries/File_manager",
                File_manager::File_manager_executable_type::new(virtual_file_system, task)
                    .await
                    .unwrap()
            ),
            (
                &"/Binaries/Command_line_shell",
                Command_line_shell::Shell_executable_type
            ),
            (
                &"/Binaries/Terminal",
                Terminal::Terminal_executable_type::new(Virtual_file_system::get_instance(), task)
                    .await
                    .unwrap()
            ),
            (
                &"/Binaries/Settings",
                Settings::Settings_executable_type::new(virtual_file_system, task)
                    .await
                    .unwrap()
            ),
            (&"/Binaries/WASM", WASM::WASM_device_type)
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
        Virtual_file_system::get_instance(),
    )
    .await
    .unwrap();

    // - - Create the default user
    let group_identifier = Users::Group_identifier_type::New(1000);

    let _ = Authentication::create_group(
        Virtual_file_system::get_instance(),
        "alix_anneraud",
        Some(group_identifier),
    )
    .await;

    let _ = Authentication::create_user(
        Virtual_file_system::get_instance(),
        "alix_anneraud",
        "password",
        group_identifier,
        None,
    )
    .await;

    // - - Set the environment variables
    task_manager
        .Set_environment_variable(task, "Paths", "/")
        .await
        .unwrap();

    task_manager
        .Set_environment_variable(task, "Host", "xila")
        .await
        .unwrap();
    // - - Execute the shell
    let _ = Executable::execute("/Binaries/Graphical_shell", String::from(""), standard)
        .await
        .unwrap()
        .Join()
        .await;

    Virtual_file_system::get_instance().uninitialize().await;
}
