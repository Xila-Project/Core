use file_system::{DirectCharacterDevice, MemoryDevice};
use users::GroupIdentifier;
use virtual_file_system::create_default_hierarchy;

pub async fn initialize_for_tests(
    logger: &'static impl log::LoggerTrait,
    time_device: &'static impl DirectCharacterDevice,
    (screen_device, pointer_device, keyboard_device): (
        &'static impl DirectCharacterDevice,
        &'static impl DirectCharacterDevice,
        &'static impl DirectCharacterDevice,
    ),
) {
    log::initialize(logger).unwrap();

    let task_manager = task::initialize();
    let users = users::initialize();
    let time = time::initialize(time_device).unwrap();
    let memory_device = MemoryDevice::<512>::new_static(1024 * 512);

    let file_system = little_fs::FileSystem::new_format(memory_device, 256).unwrap();

    let virtual_file_system =
        virtual_file_system::initialize(task_manager, users, time, file_system, None).unwrap();

    let task = task_manager.get_current_task_identifier().await;

    create_default_hierarchy(virtual_file_system, task)
        .await
        .unwrap();

    let group_identifier = GroupIdentifier::new(1000);

    authentication::create_group(virtual_file_system, "alix_anneraud", Some(group_identifier))
        .await
        .unwrap();

    authentication::create_user(
        virtual_file_system,
        "alix_anneraud",
        "password",
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
