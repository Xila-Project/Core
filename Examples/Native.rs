#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use Executable::Mount_static_executables;
use Executable::Standard_type;
use File_system::{Create_device, Create_file_system};
use Virtual_file_system::Mount_static_devices;

fn main() {
    // - Initialize the system
    // Initialize the task manager
    Task::Initialize().unwrap();
    // Initialize the users manager
    Users::Initialize().unwrap();
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
    );

    Graphics::Get_instance()
        .Add_input_device(Keyboard_device, Graphics::Input_type_type::Keypad)
        .unwrap();

    // - Initialize the file system
    // Create a memory device
    let Drive = Create_device!(Drivers::Native::File_drive_device_type::New(&"./Drive"));
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
    Virtual_file_system::Initialize(Create_file_system!(File_system)).unwrap();

    // - - Mount the devices
    let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

    // - - Create the default system hierarchy
    let _ =
        Virtual_file_system::Create_default_hierarchy(Virtual_file_system::Get_instance(), Task);

    // - - Mount the devices
    Virtual_file_system::Clean_devices(Virtual_file_system::Get_instance()).unwrap();

    Mount_static_devices!(
        Virtual_file_system::Get_instance(),
        Task,
        &[
            (
                &"/Devices/Standard_in",
                Drivers::Native::Console::Standard_in_device_type
            ),
            (
                &"/Devices/Standard_out",
                Drivers::Native::Console::Standard_out_device_type
            ),
            (
                &"/Devices/Standard_error",
                Drivers::Native::Console::Standard_error_device_type
            ),
            (&"/Devices/Time", Drivers::Native::Time_driver_type),
            (&"/Devices/Random", Drivers::Native::Random_device_type),
            (&"/Devices/Null", Drivers::Common::Null_device_type)
        ]
    )
    .unwrap();

    // Initialize the virtual machine
    Virtual_machine::Initialize(&[&Host_bindings::Graphics_bindings]);

    // Mount static executables

    let Virtual_file_system = Virtual_file_system::Get_instance();

    Mount_static_executables!(
        Virtual_file_system,
        Task,
        &[
            Graphical_shell::Shell_executable_type,
            Command_line_shell::Shell_executable_type,
            Terminal::Terminal_executable_type,
            WASM::WASM_device_type
        ]
    )
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
    .unwrap();

    // - - Create the default user
    let Group_identifier = Users::Group_identifier_type::New(1000);

    let _ = Authentication::Create_group(
        Virtual_file_system::Get_instance(),
        "alix_anneraud",
        Some(Group_identifier),
    );

    let _ = Authentication::Create_user(
        Virtual_file_system::Get_instance(),
        "alix_anneraud",
        "password",
        Group_identifier,
        None,
    );

    // - - Set the environment variables
    Task::Get_instance()
        .Set_environment_variable(Task, "Paths", "/")
        .unwrap();

    Task::Get_instance()
        .Set_environment_variable(Task, "Host", "xila")
        .unwrap();
    // - - Execute the shell
    let _ = Executable::Execute("/Binaries/Graphical_shell", "".to_string(), Standard)
        .unwrap()
        .Join()
        .unwrap();

    Virtual_file_system::Uninitialize();
}
