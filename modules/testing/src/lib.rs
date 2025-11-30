#![no_std]

extern crate alloc;

extern crate abi_definitions;

use alloc::boxed::Box;
use drivers_native::window_screen;
use drivers_shared::devices::RandomDevice;
use drivers_std::{devices::TimeDevice, log::Logger};
use executable::Standard;
use file_system::MemoryDevice;
use users::GroupIdentifier;
use virtual_file_system::{ItemStatic, create_default_hierarchy, mount_static};

pub async fn initialize(graphics_enabled: bool) -> Standard {
    log::initialize(&Logger).unwrap();

    let task_manager = task::initialize();
    let users = users::initialize();
    let time = time::initialize(&TimeDevice).unwrap();

    if graphics_enabled {
        let (screen_device, pointer_device, keyboard_device, mut runner) =
            window_screen::new(graphics::Point::new(800, 600))
                .await
                .unwrap();

        let graphics_manager = graphics::initialize(
            Box::leak(Box::new(screen_device)),
            Box::leak(Box::new(pointer_device)),
            graphics::InputKind::Pointer,
            1024 * 512,
            true,
        )
        .await;

        graphics_manager
            .add_input_device(
                Box::leak(Box::new(keyboard_device)),
                graphics::InputKind::Keypad,
            )
            .await
            .unwrap();

        task_manager
            .spawn(
                task::get_instance().get_current_task_identifier().await,
                "Graphics",
                None,
                |_| graphics_manager.r#loop(task::Manager::sleep),
            )
            .await
            .unwrap();

        task_manager
            .spawn(
                task::get_instance().get_current_task_identifier().await,
                "Window screen runner",
                None,
                async move |_| {
                    runner.run().await;
                },
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
            ItemStatic::CharacterDevice(&RandomDevice),
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

    mount_static!(
        virtual_file_system,
        task,
        &[
            (
                &"/devices/standard_in",
                CharacterDevice,
                drivers_std::console::StandardInDevice
            ),
            (
                &"/devices/standard_out",
                CharacterDevice,
                drivers_std::console::StandardOutDevice
            ),
            (
                &"/devices/standard_error",
                CharacterDevice,
                drivers_std::console::StandardErrorDevice
            ),
            (
                &"/devices/time",
                CharacterDevice,
                drivers_std::devices::TimeDevice
            ),
            (&"/devices/null", CharacterDevice, drivers_core::NullDevice),
            (
                &"/devices/hasher",
                CharacterDevice,
                drivers_shared::devices::HashDevice
            ),
        ]
    )
    .await
    .unwrap();

    Standard::open(
        &"/devices/standard_in",
        &"/devices/standard_out",
        &"/devices/standard_error",
        task,
        virtual_file_system,
    )
    .await
    .unwrap()
}
