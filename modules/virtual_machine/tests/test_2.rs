extern crate alloc;
extern crate std;

extern crate abi_definitions;

use std::fs;

use alloc::vec;

use executable::build_crate;
use wamr_rust_sdk::value::WasmValue;

use file_system::{MemoryDevice, create_device, create_file_system};
use task::test;
use virtual_file_system::{create_default_hierarchy, mount_static_devices};
use virtual_machine::{
    Environment, Function_descriptors, FunctionDescriptor, Instance, Module, Registrable, Runtime,
};

drivers::standard_library::memory::instantiate_global_allocator!();

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

    static LOGGER: drivers::standard_library::log::Logger = drivers::standard_library::log::Logger;

    log::initialize(&LOGGER).expect("Failed to initialize logger");

    users::initialize();

    time::initialize(create_device!(drivers::native::TimeDriver::new()))
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
                drivers::standard_library::console::StandardInDevice
            ),
            (
                &"/devices/standard_out",
                drivers::standard_library::console::StandardOutDevice
            ),
            (
                &"/devices/standard_error",
                drivers::standard_library::console::StandardErrorDevice
            ),
            (&"/devices/time", drivers::native::TimeDriver),
            (&"/devices/random", drivers::shared::devices::RandomDevice),
            (&"/devices/null", drivers::core::NullDevice)
        ]
    )
    .await
    .unwrap();

    task_instance
        .set_environment_variable(task, "Path", "/:/bin:/usr/bin")
        .await
        .unwrap();

    let wasm_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("./tests/wasm_test");

    let binary_path = build_crate(&wasm_path).unwrap();

    let binary_buffer = fs::read(&binary_path).expect("Failed to read the binary file");

    let standard_in = virtual_file_system
        .open(
            &"/devices/standard_in",
            file_system::Mode::READ_ONLY.into(),
            task,
        )
        .await
        .expect("Failed to open stdin");
    let standard_out = virtual_file_system
        .open(
            &"/devices/standard_out",
            file_system::Mode::WRITE_ONLY.into(),
            task,
        )
        .await
        .expect("Failed to open stdout");
    let standard_error = virtual_file_system
        .open(
            &"/devices/standard_error",
            file_system::Mode::WRITE_ONLY.into(),
            task,
        )
        .await
        .expect("Failed to open stderr");

    let (standard_in, standard_out, standard_error) = virtual_file_system
        .create_new_task_standard_io(standard_in, standard_error, standard_out, task, task, false)
        .await
        .unwrap();

    let virtual_machine = virtual_machine::initialize(&[&WasmTest]);

    virtual_machine
        .execute(
            binary_buffer.to_vec(),
            4 * 1024,
            standard_in,
            standard_out,
            standard_error,
        )
        .await
        .unwrap();
}
