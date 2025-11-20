use alloc::boxed::Box;
use file_system::{DirectCharacterDevice, MemoryDevice};
use users::GroupIdentifier;
use virtual_file_system::{ItemStatic, create_default_hierarchy};

pub async fn initialize_for_tests(
    logger: &'static impl log::LoggerTrait,
    time_device: &'static impl DirectCharacterDevice,
    random_device: &'static impl DirectCharacterDevice,
    screen_pointer_devices: Option<(
        Box<dyn DirectCharacterDevice + 'static>,
        Box<dyn DirectCharacterDevice + 'static>,
    )>,
    keyboard_device: Option<Box<dyn DirectCharacterDevice + 'static>>,
) {
    log::initialize(logger).unwrap();

    let task_manager = task::initialize();
    let users = users::initialize();
    let time = time::initialize(time_device).unwrap();

    if let Some((screen_device, pointer_device)) = screen_pointer_devices {
        let graphics_manager = graphics::initialize(
            Box::leak(screen_device),
            Box::leak(pointer_device),
            graphics::InputKind::Pointer,
            1024 * 512,
            true,
        )
        .await;

        if let Some(keyboard_device) = keyboard_device {
            graphics_manager
                .add_input_device(Box::leak(keyboard_device), graphics::InputKind::Keypad)
                .await
                .unwrap();
        }

        task_manager
            .spawn(
                task::get_instance().get_current_task_identifier().await,
                "Graphics",
                None,
                |_| graphics_manager.r#loop(task::Manager::sleep),
            )
            .await
            .unwrap();
    }

    let memory_device = MemoryDevice::<512>::new_static(1024 * 512);

    let file_system = little_fs::FileSystem::new_format(memory_device, 256).unwrap();

    let virtual_file_system =
        virtual_file_system::initialize(task_manager, users, time, file_system, None).unwrap();

    let task = task_manager.get_current_task_identifier().await;

    create_default_hierarchy(virtual_file_system, task)
        .await
        .unwrap();

    virtual_file_system
        .mount_static(
            task,
            &"/devices/random",
            ItemStatic::CharacterDevice(random_device),
        )
        .await
        .unwrap();

    let group_identifier = GroupIdentifier::new(1000);

    authentication::create_group(virtual_file_system, "administrator", Some(group_identifier))
        .await
        .unwrap();

    authentication::create_user(
        virtual_file_system,
        "administrator",
        "",
        group_identifier,
        None,
    )
    .await
    .unwrap();

    task_manager
        .set_environment_variable(task, "Paths", "/")
        .await
        .unwrap();

    task_manager
        .set_environment_variable(task, "Host", "xila")
        .await
        .unwrap();
}
