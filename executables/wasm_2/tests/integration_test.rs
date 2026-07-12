#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[xila::task::test(task_path = xila::task)]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate alloc;
    extern crate abi_definitions;

    use command_line_shell::ShellExecutable;
    use drivers_std::loader::load_to_virtual_file_system;
    use wasm_2::WasmExecutable;
    use xila::executable::{Standard, build_crate, mount_executables};
    use xila::file_system::Path;
    use xila::virtual_file_system;
    use xila::virtual_file_system::File;
    use xila::{log, task};

    let _ = testing::initialize(false, false).await;

    let virtual_file_system = virtual_file_system::get_instance();
    let task_instance = task::get_instance();
    let task = task_instance.get_current_task_identifier().await;

    let standard = Standard::open(
        &"/devices/standard_in",
        &"/devices/standard_out",
        &"/devices/standard_error",
        task,
        virtual_file_system,
    )
    .await
    .unwrap();

    let binary_path = build_crate("wasm_wasm_test_2").unwrap();
    load_to_virtual_file_system(virtual_file_system, binary_path, "/test_wasm.wasm")
        .await
        .unwrap();

    mount_executables!(
        virtual_file_system,
        task,
        &[
            ("/binaries/command_line_shell", ShellExecutable),
            ("/binaries/wasm_2", WasmExecutable),
        ]
    )
    .await
    .unwrap();

    log::information!("Executing wasm_2 test...");

    let result = executable::execute(
        "/binaries/wasm_2",
        vec!["/test_wasm.wasm".to_string()],
        standard,
        None,
    )
    .await
    .unwrap()
    .join()
    .await;

    assert!(result == 0);

    let mut contents = Vec::new();
    File::read_from_path(
        virtual_file_system,
        task,
        &Path::new("/test.txt"),
        &mut contents,
    )
    .await
    .unwrap();
    assert_eq!(contents, b"Hello World from WASM!");
}
