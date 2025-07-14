extern crate alloc;

use alloc::string::ToString;
use command_line_shell::ShellExecutable;
use executable::{mount_static_executables, Standard};
use file_system::{create_device, create_file_system, MemoryDevice, Mode};
use task::test;
use users::GroupIdentifier;
use virtual_file_system::{create_default_hierarchy, Mount_static_devices};

#[ignore]
#[test]
async fn integration_test() {
    let task_instance = task::initialize();

    let _ = users::initialize();

    let _ = time::initialize(create_device!(drivers::native::TimeDriverType::new()));

    let memory_device = create_device!(MemoryDevice::<512>::new(1024 * 512));

    little_fs::FileSystem::format(memory_device.clone(), 256).unwrap();

    let file_system = little_fs::FileSystem::new(memory_device, 256).unwrap();

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
                drivers::standard_library::console::StandardOutDeviceType
            ),
            (
                &"/devices/Standard_error",
                drivers::standard_library::console::StandardErrorDeviceType
            ),
            (&"/devices/Time", drivers::native::TimeDriverType),
            (&"/devices/Random", drivers::native::RandomDeviceType),
            (&"/devices/Null", drivers::core::NullDeviceType)
        ]
    )
    .await
    .unwrap();

    mount_static_executables!(
        virtual_file_system,
        task,
        &[(&"/binaries/Command_line_shell", ShellExecutable)]
    )
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

    task_instance
        .set_environment_variable(task, "Paths", "/")
        .await
        .unwrap();

    task_instance
        .set_environment_variable(task, "Host", "xila")
        .await
        .unwrap();

    let result = executable::execute("/binaries/Command_line_shell", "".to_string(), standard)
        .await
        .unwrap()
        .join()
        .await;

    assert!(result == 0);
}
