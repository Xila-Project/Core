#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[xila::task::test(task_path = xila::task)]
#[ignore = "This test is meant to be run interactively"]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate alloc;

    use drivers_std::loader::load_to_virtual_file_system;
    use wasm::WasmExecutable;
    use xila::executable;
    use xila::executable::{build_crate, mount_executables};
    use xila::task;
    use xila::virtual_file_system;

    let standard = testing::initialize(true, false).await;

    let virtual_file_system = virtual_file_system::get_instance();
    let task_instance = task::get_instance();
    let task = task_instance.get_current_task_identifier().await;

    let binary_path = build_crate(&"calculator").unwrap();
    load_to_virtual_file_system(
        virtual_file_system,
        binary_path,
        "/binaries/calculator.wasm",
    )
    .await
    .unwrap();

    fn new_thread_executor_wrapper()
    -> core::pin::Pin<Box<dyn Future<Output = task::SpawnerIdentifier> + Send>> {
        use drivers_std::executor::new_thread_executor;

        Box::pin(new_thread_executor())
    }

    mount_executables!(
        virtual_file_system,
        task,
        &[(
            "/binaries/wasm",
            WasmExecutable::new(Some(new_thread_executor_wrapper))
        )]
    )
    .await
    .unwrap();

    let result = executable::execute(
        "/binaries/wasm",
        vec!["/binaries/calculator.wasm".to_string()],
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
