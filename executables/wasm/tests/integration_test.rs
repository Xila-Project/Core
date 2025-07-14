extern crate alloc;

use command_line_shell::ShellExecutable;
use drivers::standard_library::loader::Loader;
use executable::{mount_static_executables, Standard};
use file_system::{create_device, create_file_system, MemoryDevice, Mode};
use memory::instantiate_global_allocator;
use task::test;
use virtual_file_system::{create_default_hierarchy, Mount_static_devices};
use wasm::WasmDevice;

instantiate_global_allocator!(drivers::standard_library::memory::MemoryManager);

#[ignore]
#[test]
async fn i() {
    let task_instance = task::initialize();

    let _ = users::initialize();

    let _ = time::initialize(create_device!(drivers::native::TimeDriver::new()));

    let _ = virtual_machine::initialize(&[]);

    let memory_device = create_device!(MemoryDevice::<512>::new(1024 * 1024 * 512));

    little_fs::FileSystem::format(memory_device.clone(), 256).unwrap();

    let mut file_system = little_fs::FileSystem::new(memory_device, 256).unwrap();

    let wasm_executable_path = "./tests/wasm_test/target/wasm32-wasip1/release/WASM_test.wasm";
    let destination = "/test.wasm";

    Loader::new()
        .add_file(wasm_executable_path, destination)
        .load(&mut file_system)
        .unwrap();

    let virtual_file_system =
        virtual_file_system::initialize(create_file_system!(file_system), None).unwrap();

    let task = task_instance.get_current_task_identifier().await;

    create_default_hierarchy(virtual_file_system, task)
        .await
        .unwrap();

    Mount_static_devices!(
        virtual_file_system,
        task,
        &[
            (
                &"/devices/Standard_in",
                drivers::standard_library::console::StandardInDevice
            ),
            (
                &"/devices/Standard_out",
                drivers::standard_library::console::StandardOutDevice
            ),
            (
                &"/devices/Standard_error",
                drivers::standard_library::console::StandardErrorDevice
            ),
            (&"/devices/Time", drivers::native::TimeDriver),
            (&"/devices/Random", drivers::native::RandomDevice),
            (&"/devices/Null", drivers::core::NullDevice)
        ]
    )
    .await
    .unwrap();

    mount_static_executables!(
        virtual_file_system,
        task,
        &[
            (&"/binaries/Command_line_shell", ShellExecutable),
            (&"/binaries/WASM", WasmDevice)
        ]
    )
    .await
    .unwrap();

    let standard_in = virtual_file_system
        .open(&"/devices/Standard_in", Mode::READ_ONLY.into(), task)
        .await
        .unwrap();

    let standard_out = virtual_file_system
        .open(&"/devices/Standard_out", Mode::WRITE_ONLY.into(), task)
        .await
        .unwrap();

    let standard_error = virtual_file_system
        .open(&"/devices/Standard_error", Mode::WRITE_ONLY.into(), task)
        .await
        .unwrap();

    let standard = Standard::new(
        standard_in,
        standard_out,
        standard_error,
        task,
        virtual_file_system,
    );

    let environment_variables = &[("Paths", "/"), ("User", "alix_anneraud"), ("Host", "xila")];

    task_instance
        .set_environment_variables(task, environment_variables)
        .await
        .unwrap();

    let result = executable::execute("/binaries/Command_line_shell", "".to_string(), standard)
        .await
        .unwrap()
        .join()
        .await;

    assert!(result == 0);
}
