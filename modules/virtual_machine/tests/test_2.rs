extern crate alloc;
extern crate std;

extern crate abi_definitions;

use std::fs;

use executable::{Standard, build_crate};

use file_system::{MemoryDevice, create_device, create_file_system};
use task::test;
use virtual_file_system::{create_default_hierarchy, mount_static_devices};
use virtual_machine::{Function_descriptors, FunctionDescriptor, Registrable};

drivers_std::memory::instantiate_global_allocator!();

pub struct WasmTest;

impl Registrable for WasmTest {
    fn get_functions(&self) -> &[FunctionDescriptor] {
        &FUNCTIONS
    }

    fn get_name(&self) -> &'static str {
        "Virtual_machine_WASM_test"
    }
}

const FUNCTIONS: [FunctionDescriptor; 0] = Function_descriptors! {};

#[ignore]
#[test]
async fn integration_test_2() {
    let task_instance = task::initialize();

    static LOGGER: drivers_std::log::Logger = drivers_std::log::Logger;

    log::initialize(&LOGGER).expect("Failed to initialize logger");

    users::initialize();

    time::initialize(create_device!(drivers_native::TimeDriver::new()))
        .expect("Failed to initialize time manager");

    let device = create_device!(MemoryDevice::<512>::new(1024 * 512));

    little_fs::FileSystem::format(device.clone(), 512).unwrap();
    let file_system = create_file_system!(little_fs::FileSystem::new(device, 256).unwrap());

    let virtual_file_system = virtual_file_system::initialize(file_system, None).unwrap();

    // Set environment variables
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

    task_instance
        .set_environment_variable(task, "Path", "/:/bin:/usr/bin")
        .await
        .unwrap();

    let binary_path = build_crate(&"virtual_machine_wasm_test").unwrap();

    let binary_buffer = fs::read(&binary_path).expect("Failed to read the binary file");

    let standard = Standard::open(
        &"/devices/standard_in",
        &"/devices/standard_out",
        &"/devices/standard_error",
        task,
        virtual_file_system,
    )
    .await
    .unwrap()
    .split();

    let virtual_machine = virtual_machine::initialize(&[&WasmTest]);

    virtual_machine
        .execute(
            binary_buffer.to_vec(),
            4 * 1024,
            standard,
            None,
            vec![],
            task,
        )
        .await
        .unwrap();
}
