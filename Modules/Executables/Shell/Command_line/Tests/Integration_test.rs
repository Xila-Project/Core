#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate alloc;

use alloc::string::ToString;
use Command_line_shell::Shell_executable_type;
use Executable::Standard_type;
use File_system::{Create_device, Create_file_system, Memory_device_type, Mode_type};
use Task::Test;
use Users::Group_identifier_type;

#[ignore]
#[Test]
async fn Integration_test() {
    let Task_instance = Task::Initialize();

    let _ = Users::Initialize();

    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

    let Memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

    LittleFS::File_system_type::Format(Memory_device.clone(), 256).unwrap();

    let File_system = LittleFS::File_system_type::New(Memory_device, 256).unwrap();

    Virtual_file_system::Initialize(Create_file_system!(File_system), None).unwrap();

    let Task = Task_instance.Get_current_task_identifier().await;

    Virtual_file_system::Get_instance()
        .Mount_static_device(Task, &"/Shell", Create_device!(Shell_executable_type))
        .await
        .unwrap();

    Virtual_file_system::Get_instance()
        .Create_directory(&"/Devices", Task)
        .await
        .unwrap();

    Virtual_file_system::Get_instance()
        .Create_directory(&"/System", Task)
        .await
        .unwrap();

    Virtual_file_system::Get_instance()
        .Create_directory(&"/System/Users", Task)
        .await
        .unwrap();

    Virtual_file_system::Get_instance()
        .Create_directory(&"/System/Groups", Task)
        .await
        .unwrap();

    Virtual_file_system::Get_instance()
        .Mount_static_device(
            Task,
            &"/Devices/Random",
            Create_device!(Drivers::Native::Random_device_type),
        )
        .await
        .unwrap();

    let Group_identifier = Group_identifier_type::New(1000);

    Authentication::Create_group(
        Virtual_file_system::Get_instance(),
        "alix_anneraud",
        Some(Group_identifier),
    )
    .await
    .unwrap();

    Authentication::Create_user(
        Virtual_file_system::Get_instance(),
        "alix_anneraud",
        "password",
        Group_identifier,
        None,
    )
    .await
    .unwrap();

    Drivers::Native::Console::Mount_devices(Task, Virtual_file_system::Get_instance())
        .await
        .unwrap();

    let Standard_in = Virtual_file_system::Get_instance()
        .Open(&"/Devices/Standard_in", Mode_type::Read_only.into(), Task)
        .await
        .unwrap();

    let Standard_out = Virtual_file_system::Get_instance()
        .Open(&"/Devices/Standard_out", Mode_type::Write_only.into(), Task)
        .await
        .unwrap();

    let Standard_error = Virtual_file_system::Get_instance()
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
        Virtual_file_system::Get_instance(),
    );

    Task_instance
        .Set_environment_variable(Task, "Paths", "/")
        .await
        .unwrap();

    Task_instance
        .Set_environment_variable(Task, "Host", "xila")
        .await
        .unwrap();

    let Result = Executable::Execute("/Shell", "".to_string(), Standard)
        .await
        .unwrap()
        .Join()
        .await;

    assert!(Result == 0);
}
