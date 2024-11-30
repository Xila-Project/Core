#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use Command_line_shell::Shell_executable_type;
use Executable::Standard_type;
use File_system::{Create_device, Create_file_system, Mode_type};
use WASM::WASM_device_type;

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
    let (Screen, Pointer) = Drivers::Native::Window_screen::New(Resolution).unwrap();
    // - - Initialize the graphics manager
    Graphics::Initialize();
    // - - Add a screen
    const Buffer_size: usize = Graphics::Get_recommended_buffer_size(&Resolution);
    let _ = Graphics::Get_instance()
        .Create_display::<Buffer_size>(Screen, Pointer, true)
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
    Virtual_file_system::Initialize(Create_file_system!(File_system));

    // - - Mount the devices
    let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

    Virtual_file_system::Get_instance()
        .Mount_static_device(Task, &"/Shell", Create_device!(Shell_executable_type))
        .unwrap();

    Virtual_file_system::Get_instance()
        .Mount_static_device(Task, &"/WASM", Create_device!(WASM_device_type))
        .unwrap();

    let _ = Virtual_file_system::Get_instance().Create_directory(&"/Devices", Task);

    Drivers::Native::Console::Mount_devices(Task, Virtual_file_system::Get_instance()).unwrap();
    // Initialize the virtual machine
    Virtual_machine::Initialize(&[&Host_bindings::Graphics_bindings]);

    // - Execute the shell
    // - - Open the standard input, output and error
    let Standard_in = Virtual_file_system::Get_instance()
        .Open(&"/Devices/Standard_in", Mode_type::Read_only.into(), Task)
        .unwrap();

    let Standard_out = Virtual_file_system::Get_instance()
        .Open(&"/Devices/Standard_out", Mode_type::Write_only.into(), Task)
        .unwrap();

    let Standard_error = Virtual_file_system::Get_instance()
        .Open(
            &"/Devices/Standard_error",
            Mode_type::Write_only.into(),
            Task,
        )
        .unwrap();

    let Standard = Standard_type::New(
        Standard_in,
        Standard_out,
        Standard_error,
        Task,
        Virtual_file_system::Get_instance(),
    );
    // - - Set the environment variables
    Task::Get_instance()
        .Set_environment_variable(Task, "Paths", "/")
        .unwrap();

    Task::Get_instance()
        .Set_environment_variable(Task, "Host", "xila")
        .unwrap();
    // - - Execute the shell
    let _ = Executable::Execute("/Shell", "".to_string(), Standard)
        .unwrap()
        .Join()
        .unwrap();

    Virtual_file_system::Uninitialize();
}
