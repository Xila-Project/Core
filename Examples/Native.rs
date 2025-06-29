#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate alloc;

use alloc::string::String;
use Executable::Mount_static_executables;
use Executable::Standard_type;
use File_system::{Create_device, Create_file_system};
use Memory::Instantiate_global_allocator;
use Virtual_file_system::Mount_static_devices;

Instantiate_global_allocator!(Drivers::Std::Memory::Memory_manager_type);

#[Task::Run_with_executor(Drivers::Std::Executor::Executor_type)]
async fn main() {
    // - Initialize the system
    Log::Initialize(&Drivers::Std::Log::Logger_type).unwrap();

    // Initialize the task manager
    Task::Initialize();

    // Initialize the users manager
    Users::Initialize();
    // Initialize the time manager
    Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New())).unwrap();

    // - Initialize the graphics manager
    // - - Initialize the graphics driver
    const Resolution: Graphics::Point_type = Graphics::Point_type::New(800, 600);
    let (Screen_device, Pointer_device, Keyboard_device) =
        Drivers::Native::Window_screen::New(Resolution).unwrap();
    // - - Initialize the graphics manager
    Graphics::Initialize(
        Screen_device,
        Pointer_device,
        Graphics::Input_type_type::Pointer,
        Graphics::Get_minimal_buffer_size(&Resolution),
        true,
    )
    .await;

    Graphics::Get_instance()
        .Add_input_device(Keyboard_device, Graphics::Input_type_type::Keypad)
        .await
        .unwrap();

    // - Initialize the file system
    // Create a memory device
    let Drive = Create_device!(Drivers::Std::Drive_file::File_drive_device_type::New(
        &"./Drive.img"
    ));
    // Mount the file system
    let File_system = match LittleFS::File_system_type::New(Drive.clone(), 256) {
        Ok(File_system) => File_system,
        // If the file system is not found, format it
        Err(_) => {
            Drive
                .Set_position(&File_system::Position_type::Start(0))
                .unwrap();

            LittleFS::File_system_type::Format(Drive.clone(), 256).unwrap();

            LittleFS::File_system_type::New(Drive, 256).unwrap()
        }
    };
    // Initialize the virtual file system
    Virtual_file_system::Initialize(Create_file_system!(File_system), None).unwrap();

    // - - Mount the devices
    let Task = Task::Get_instance().Get_current_task_identifier().await;

    // - - Create the default system hierarchy
    let _ =
        Virtual_file_system::Create_default_hierarchy(Virtual_file_system::Get_instance(), Task)
            .await;

    // - - Mount the devices
    Virtual_file_system::Clean_devices(Virtual_file_system::Get_instance())
        .await
        .unwrap();

    Mount_static_devices!(
        Virtual_file_system::Get_instance(),
        Task,
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

    let Virtual_file_system = Virtual_file_system::Get_instance();

    Mount_static_executables!(
        Virtual_file_system,
        Task,
        &[
            (
                &"/Binaries/Graphical_shell",
                Graphical_shell::Shell_executable_type
            ),
            (
                &"/Binaries/File_manager",
                File_manager::File_manager_executable_type::New(Virtual_file_system, Task)
                    .await
                    .unwrap()
            ),
            (
                &"/Binaries/Command_line_shell",
                Command_line_shell::Shell_executable_type
            ),
            (
                &"/Binaries/Terminal",
                Terminal::Terminal_executable_type::New(Virtual_file_system::Get_instance(), Task)
                    .await
                    .unwrap()
            ),
            (
                &"/Binaries/Settings",
                Settings::Settings_executable_type::New(Virtual_file_system, Task)
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
    let Standard = Standard_type::Open(
        &"/Devices/Standard_in",
        &"/Devices/Standard_out",
        &"/Devices/Standard_error",
        Task,
        Virtual_file_system::Get_instance(),
    )
    .await
    .unwrap();

    // - - Create the default user
    let Group_identifier = Users::Group_identifier_type::New(1000);

    let _ = Authentication::Create_group(
        Virtual_file_system::Get_instance(),
        "alix_anneraud",
        Some(Group_identifier),
    )
    .await;

    let _ = Authentication::Create_user(
        Virtual_file_system::Get_instance(),
        "alix_anneraud",
        "password",
        Group_identifier,
        None,
    )
    .await;

    // - - Set the environment variables
    Task::Get_instance()
        .Set_environment_variable(Task, "Paths", "/")
        .await
        .unwrap();

    Task::Get_instance()
        .Set_environment_variable(Task, "Host", "xila")
        .await
        .unwrap();
    // - - Execute the shell
    let _ = Executable::Execute("/Binaries/Graphical_shell", String::from(""), Standard)
        .await
        .unwrap()
        .Join()
        .await;

    Virtual_file_system::Get_instance().Uninitialize().await;
}
