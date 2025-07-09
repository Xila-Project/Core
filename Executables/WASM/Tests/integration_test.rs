#![allow(non_camel_case_types)]

extern crate alloc;

use command_line_shell::Shell_executable_type;
use drivers::std::Loader::Loader_type;
use executable::{Mount_static_executables, Standard_type};
use file_system::{Create_device, Create_file_system, Memory_device_type, Mode_type};
use memory::Instantiate_global_allocator;
use task::Test;
use virtual_file_system::{create_default_hierarchy, Mount_static_devices};
use wasm::WASM_device_type;

Instantiate_global_allocator!(drivers::standard_library::memory::Memory_manager_type);

#[ignore]
#[Test]
async fn i() {
    let task_instance = task::Initialize();

    let _ = users::Initialize();

    let _ = time::Initialize(Create_device!(drivers::native::Time_driver_type::new()));

    let _ = Virtual_machine::Initialize(&[]);

    let memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 1024 * 512));

    little_fs::File_system_type::format(memory_device.clone(), 256).unwrap();

    let mut file_system = little_fs::File_system_type::new(memory_device, 256).unwrap();

    let wasm_executable_path = "./Tests/WASM_test/target/wasm32-wasip1/release/WASM_test.wasm";
    let destination = "/Test.wasm";

    Loader_type::new()
        .add_file(wasm_executable_path, destination)
        .load(&mut file_system)
        .unwrap();

    let virtual_file_system =
        virtual_file_system::initialize(Create_file_system!(file_system), None).unwrap();

    let task = task_instance.get_current_task_identifier().await;

    create_default_hierarchy(virtual_file_system, task)
        .await
        .unwrap();

    Mount_static_devices!(
        virtual_file_system,
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

    Mount_static_executables!(
        virtual_file_system,
        task,
        &[
            (&"/Binaries/Command_line_shell", Shell_executable_type),
            (&"/Binaries/WASM", WASM_device_type)
        ]
    )
    .await
    .unwrap();

    let standard_in = virtual_file_system
        .open(&"/Devices/Standard_in", Mode_type::READ_ONLY.into(), task)
        .await
        .unwrap();

    let standard_out = virtual_file_system
        .open(&"/Devices/Standard_out", Mode_type::WRITE_ONLY.into(), task)
        .await
        .unwrap();

    let standard_error = virtual_file_system
        .open(
            &"/Devices/Standard_error",
            Mode_type::WRITE_ONLY.into(),
            task,
        )
        .await
        .unwrap();

    let standard = Standard_type::new(
        standard_in,
        standard_out,
        standard_error,
        task,
        virtual_file_system,
    );

    let environment_variables = &[("Paths", "/"), ("User", "alix_anneraud"), ("Host", "xila")];

    task_instance
        .Set_environment_variables(task, environment_variables)
        .await
        .unwrap();

    let result = executable::execute("/Binaries/Command_line_shell", "".to_string(), standard)
        .await
        .unwrap()
        .Join()
        .await;

    assert!(result == 0);
}
