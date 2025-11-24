#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[xila::task::test(task_path = xila::task)]
#[ignore]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate alloc;

    use drivers_std::executor::new_thread_executor;
    use std::fs;
    use xila::executable::build_crate;
    use xila::host_bindings;
    use xila::task;
    use xila::time::Duration;
    use xila::virtual_machine;

    let binary_path = build_crate("calculator").unwrap();
    let binary_buffer = fs::read(binary_path).unwrap();

    let standard = testing::initialize(true).await.split();

    let task_manager = task::get_instance();
    let virtual_machine = virtual_machine::initialize(&[&host_bindings::GraphicsBindings]);

    let additional_spawner = new_thread_executor().await;

    task_manager
        .spawn(
            task_manager.get_current_task_identifier().await,
            "Runner",
            Some(additional_spawner),
            async move |task| {
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
            },
        )
        .await
        .unwrap();

    loop {
        task::Manager::sleep(Duration::from_millis(1000)).await;
    }
}
