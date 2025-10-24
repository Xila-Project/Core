#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[ignore]
#[xila::task::test(task_path = xila::task)]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate alloc;
    extern crate abi_definitions;

    use command_line_shell::ShellExecutable;
    use drivers_std::loader::Loader;
    use wasm::WasmDevice;
    use xila::executable::{Standard, build_crate, mount_static_executables};
    use xila::file_system::{MemoryDevice, Mode, Path, create_device, create_file_system};
    use xila::time;
    use xila::users;
    use xila::virtual_file_system::{self, create_default_hierarchy, mount_static_devices};
    use xila::virtual_machine;

    let task_instance = task::initialize();

    let _ = users::initialize();

    let _ = time::initialize(create_device!(drivers_native::TimeDriver::new()));

    let _ = virtual_machine::initialize(&[]);

    let memory_device = create_device!(MemoryDevice::<512>::new(1024 * 1024 * 512));

    little_fs::FileSystem::format(memory_device.clone(), 256).unwrap();

    let mut file_system = little_fs::FileSystem::new(memory_device, 256).unwrap();

    let crate_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("./tests/wasm_test");
    let binary_path = build_crate(&crate_path).unwrap();

    let destination = Path::new("/test_wasm.wasm");

    Loader::new()
        .add_file(binary_path, destination)
        .load(&mut file_system)
        .unwrap();

    let virtual_file_system =
        virtual_file_system::initialize(create_file_system!(file_system), None).unwrap();

    let task = task_instance.get_current_task_identifier().await;

    create_default_hierarchy(virtual_file_system, task)
        .await
        .unwrap();

    mount_static_devices!(
        virtual_file_system,
        task,
        &[
            (
                &"/devices/standard_in",
                drivers_std::console::StandardInDevice
            ),
            (
                &"/devices/standard_out",
                drivers_std::console::StandardOutDevice
            ),
            (
                &"/devices/standard_error",
                drivers_std::console::StandardErrorDevice
            ),
            (&"/devices/time", drivers_native::TimeDriver),
            (&"/devices/random", drivers_shared::devices::RandomDevice),
            (&"/devices/null", drivers_core::NullDevice)
        ]
    )
    .await
    .unwrap();

    mount_static_executables!(
        virtual_file_system,
        task,
        &[
            (&"/binaries/command_line_shell", ShellExecutable),
            (&"/binaries/wasm", WasmDevice)
        ]
    )
    .await
    .unwrap();

    let standard_in = virtual_file_system
        .open(&"/devices/standard_in", Mode::READ_ONLY.into(), task)
        .await
        .unwrap();

    let standard_out = virtual_file_system
        .open(&"/devices/standard_out", Mode::WRITE_ONLY.into(), task)
        .await
        .unwrap();

    let standard_error = virtual_file_system
        .open(&"/devices/standard_error", Mode::WRITE_ONLY.into(), task)
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

    let result = executable::execute("/binaries/command_line_shell", "".to_string(), standard)
        .await
        .unwrap()
        .join()
        .await;

    assert!(result == 0);
}
