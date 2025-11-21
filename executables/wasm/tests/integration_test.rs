#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[ignore]
#[xila::task::test(task_path = xila::task)]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate alloc;
    extern crate abi_definitions;

    use command_line_shell::ShellExecutable;
    use drivers_native::TimeDevice;
    use drivers_shared::devices::RandomDevice;
    use drivers_std::loader::load_to_virtual_file_system;
    use drivers_std::log::Logger;
    use executable::initialize_for_tests;
    use wasm::WasmExecutable;
    use xila::executable::{Standard, build_crate, mount_executables};
    use xila::task;
    use xila::virtual_file_system::{self, mount_static};
    use xila::virtual_machine;

    initialize_for_tests(&Logger, &TimeDevice, &RandomDevice, None, None).await;

    let virtual_file_system = virtual_file_system::get_instance();
    let task_instance = task::get_instance();
    let task = task_instance.get_current_task_identifier().await;

    let _ = virtual_machine::initialize(&[]);

    let binary_path = build_crate(&"wasm_wasm_test").unwrap();

    load_to_virtual_file_system(virtual_file_system, binary_path, "/test_wasm.wasm")
        .await
        .unwrap();

    mount_static!(
        virtual_file_system,
        task,
        &[
            (
                "/devices/standard_in",
                CharacterDevice,
                drivers_std::console::StandardInDevice
            ),
            (
                "/devices/standard_out",
                CharacterDevice,
                drivers_std::console::StandardOutDevice
            ),
            (
                "/devices/standard_error",
                CharacterDevice,
                drivers_std::console::StandardErrorDevice
            ),
            ("/devices/time", CharacterDevice, drivers_native::TimeDevice),
            (
                "/devices/random",
                CharacterDevice,
                drivers_shared::devices::RandomDevice
            ),
            ("/devices/null", CharacterDevice, drivers_core::NullDevice)
        ]
    )
    .await
    .unwrap();

    mount_executables!(
        virtual_file_system,
        task,
        &[
            ("/binaries/command_line_shell", ShellExecutable),
            ("/binaries/wasm", WasmExecutable)
        ]
    )
    .await
    .unwrap();

    let standard = Standard::open(
        &"/devices/standard_in",
        &"/devices/standard_out",
        &"/devices/standard_error",
        task,
        virtual_file_system,
    )
    .await
    .unwrap();

    let environment_variables = &[("Paths", "/"), ("User", "alix_anneraud"), ("Host", "xila")];

    task_instance
        .set_environment_variables(task, environment_variables)
        .await
        .unwrap();

    let result = executable::execute(
        "/binaries/wasm",
        vec!["/test_wasm.wasm".to_string()],
        standard,
        None,
    )
    .await
    .unwrap()
    .join()
    .await;

    //    let result = executable::execute("/binaries/command_line_shell", vec![], standard, None)
    //        .await
    //        .unwrap()
    //        .join()
    //        .await;

    assert!(result == 0);
}
