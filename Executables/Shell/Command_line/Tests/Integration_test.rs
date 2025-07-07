#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate alloc;

use alloc::string::ToString;
use Command_line_shell::Shell_executable_type;
use Executable::{Mount_static_executables, Standard_type};
use File_system::{Create_device, Create_file_system, Memory_device_type, Mode_type};
use Task::Test;
use Users::Group_identifier_type;
use Virtual_file_system::{Create_default_hierarchy, Mount_static_devices};

#[ignore]
#[Test]
async fn Integration_test() {
    let Task_instance = Task::Initialize();

    let _ = Users::Initialize();

    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

    let Memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

    LittleFS::File_system_type::Format(Memory_device.clone(), 256).unwrap();

    let File_system = LittleFS::File_system_type::New(Memory_device, 256).unwrap();

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
        &[(&"/Binaries/Command_line_shell", Shell_executable_type)]
    )
    .await
    .unwrap();

    let Group_identifier = Group_identifier_type::New(1000);

    Authentication::Create_group(Virtual_file_system, "alix_anneraud", Some(Group_identifier))
        .await
        .unwrap();

    Authentication::Create_user(
        Virtual_file_system,
        "alix_anneraud",
        "password",
        Group_identifier,
        None,
    )
    .await
    .unwrap();

    let Standard_in = Virtual_file_system
        .Open(&"/Devices/Standard_in", Mode_type::READ_ONLY.into(), Task)
        .await
        .unwrap();

    let Standard_out = Virtual_file_system
        .Open(&"/Devices/Standard_out", Mode_type::WRITE_ONLY.into(), Task)
        .await
        .unwrap();

    let Standard_error = Virtual_file_system
        .Open(
            &"/Devices/Standard_error",
            Mode_type::WRITE_ONLY.into(),
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

    Task_instance
        .Set_environment_variable(Task, "Paths", "/")
        .await
        .unwrap();

    Task_instance
        .Set_environment_variable(Task, "Host", "xila")
        .await
        .unwrap();

    let Result = Executable::Execute("/Binaries/Command_line_shell", "".to_string(), Standard)
        .await
        .unwrap()
        .Join()
        .await;

    assert!(Result == 0);
}
