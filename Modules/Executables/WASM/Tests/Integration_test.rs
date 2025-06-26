#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate alloc;

use Command_line_shell::Shell_executable_type;
use Drivers::Std::Loader::Loader_type;
use Executable::{Mount_static_executables, Standard_type};
use File_system::{Create_device, Create_file_system, Memory_device_type, Mode_type};
use Memory::Instantiate_global_allocator;
use Task::Test;
use Virtual_file_system::{Create_default_hierarchy, Mount_static_devices};
use WASM::WASM_device_type;

Instantiate_global_allocator!(Drivers::Std::Memory::Memory_manager_type);

#[ignore]
#[Test]
async fn Integration_test() {
    let Task_instance = Task::Initialize();

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
        Virtual_file_system::Initialize(Create_file_system!(File_system), None).unwrap();

    let Task = Task_instance.Get_current_task_identifier().await;

    Create_default_hierarchy(Virtual_file_system, Task)
        .await
        .unwrap();

    Mount_static_devices!(
        Virtual_file_system,
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

    Mount_static_executables!(
        Virtual_file_system,
        Task,
        &[
            (&"/Binaries/Command_line_shell", Shell_executable_type),
            (&"/Binaries/WASM", WASM_device_type)
        ]
    )
    .await
    .unwrap();

    let Standard_in = Virtual_file_system
        .Open(&"/Devices/Standard_in", Mode_type::Read_only.into(), Task)
        .await
        .unwrap();

    let Standard_out = Virtual_file_system
        .Open(&"/Devices/Standard_out", Mode_type::Write_only.into(), Task)
        .await
        .unwrap();

    let Standard_error = Virtual_file_system
        .Open(
            &"/Devices/Standard_error",
            Mode_type::Write_only.into(),
            Task,
        )
        .await
        .unwrap();

    let Standard = Standard_type::New(
        Standard_in,
        Standard_out,
        Standard_error,
        Task,
        Virtual_file_system,
    );

    let Environment_variables = &[("Paths", "/"), ("User", "alix_anneraud"), ("Host", "xila")];

    Task_instance
        .Set_environment_variables(Task, Environment_variables)
        .await
        .unwrap();

    let Result = Executable::Execute("/Binaries/Command_line_shell", "".to_string(), Standard)
        .await
        .unwrap()
        .Join()
        .await;

    assert!(Result == 0);
}
