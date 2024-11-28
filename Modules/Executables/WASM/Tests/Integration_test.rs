#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use Command_line_shell::Shell_executable_type;
use Executable::Standard_type;
use File_system::{
    Create_device, Create_file_system, Loader::Loader_type, Memory_device_type, Mode_type,
};
use WASM::WASM_device_type;

#[ignore]
#[test]
fn Integration_test() {
    let Task_instance = Task::Initialize().unwrap();

    let _ = Users::Initialize();

    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

    let _ = Virtual_machine::Initialize(&[]);

    let Memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 1024 * 512));

    LittleFS::File_system_type::Format(Memory_device.clone(), 256).unwrap();

    let mut File_system = LittleFS::File_system_type::New(Memory_device, 256).unwrap();

    let WASM_executable_path = "./Tests/WASM_test/target/wasm32-wasip1/release/WASM_test.wasm";
    let Destination = "/Test.wasm";
    Loader_type::New()
        .Add_file(WASM_executable_path, Destination)
        .Load(&mut File_system)
        .unwrap();

    let Virtual_file_system =
        Virtual_file_system::Initialize(Create_file_system!(File_system)).unwrap();

    let Task = Task_instance.Get_current_task_identifier().unwrap();

    Virtual_file_system
        .Mount_device(Task, "/Shell", Create_device!(Shell_executable_type))
        .unwrap();

    Virtual_file_system
        .Mount_device(Task, "/WASM", Create_device!(WASM_device_type))
        .unwrap();

    Virtual_file_system
        .Create_directory(&"/Devices", Task)
        .unwrap();

    Drivers::Native::Console::Mount_devices(Task, Virtual_file_system).unwrap();

    let Standard_in = Virtual_file_system
        .Open(&"/Devices/Standard_in", Mode_type::Read_only.into(), Task)
        .unwrap();

    let Standard_out = Virtual_file_system
        .Open(&"/Devices/Standard_out", Mode_type::Write_only.into(), Task)
        .unwrap();

    let Standard_error = Virtual_file_system
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
        Virtual_file_system,
    );

    Task_instance
        .Set_environment_variable(Task, "Paths", "/")
        .unwrap();

    Task_instance
        .Set_environment_variable(Task, "User", "alix_anneraud")
        .unwrap();

    Task_instance
        .Set_environment_variable(Task, "Host", "xila")
        .unwrap();

    let Result = Executable::Execute("/Shell", "".to_string(), Standard)
        .unwrap()
        .Join()
        .unwrap();

    assert!(Result == 0);
}
