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
async fn integration_test() {
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

    abi_context::get_instance()
        .call_abi(async || {
            // Register the functions

            let runtime = Runtime::builder().register(&WasmTest).build().unwrap();

            let module = Module::from_buffer(
                &runtime,
                binary_buffer.to_vec(),
                "main",
                standard_in,
                standard_out,
                standard_error,
            )
            .await
            .unwrap();

            let mut instance =
                Instance::new(&runtime, &module, 1024 * 4).expect("Failed to instantiate module");

            let _ =
                Environment::from_instance(&instance).expect("Failed to get execution environment");

            assert_eq!(instance.call_main(&vec![]).unwrap(), [WasmValue::Void]);

            assert_eq!(
                instance
                    .call_export_function("gcd", &vec![WasmValue::I32(9), WasmValue::I32(27)])
                    .unwrap(),
                [WasmValue::I32(9)]
            );

            // Test allocation and deallocation

            let pointer = instance.allocate::<u32>(4).unwrap();

            unsafe {
                pointer.write(1234);

                assert_eq!(1234, pointer.read());
            }

            instance.deallocate(pointer);
        })
        .await;
}
