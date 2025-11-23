#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[ignore]
#[task::test]
async fn test() {
    extern crate abi_definitions;
    extern crate alloc;
    extern crate std;

    use executable::build_crate;
    use std::fs;
    use time::Duration;

    drivers_std::memory::instantiate_global_allocator!();

    let binary_path = build_crate(&"host_bindings_wasm_test").unwrap();
    let binary_buffer = fs::read(&binary_path).unwrap();

    let standard = testing::initialize(true).await.split();

    let virtual_machine = virtual_machine::initialize(&[&host_bindings::GraphicsBindings]);

    let task_manager = task::get_instance();
    let task = task_manager.get_current_task_identifier().await;

    //let window = graphics_manager.create_window().await.unwrap();

    //let _calendar = unsafe { lvgl::lv_calendar_create(window.into_raw()) };

    virtual_machine
        .execute(
            binary_buffer.to_vec(),
            8 * 1024,
            standard,
            None,
            vec![],
            task,
        )
        .await
        .unwrap();

    loop {
        task::Manager::sleep(Duration::from_millis(1000)).await;
    }
}
