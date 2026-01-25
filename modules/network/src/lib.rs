#![no_std]

extern crate alloc;

mod device;
mod error;
mod fundamentals;
mod manager;
mod socket;

pub use device::*;
pub use error::*;
pub use fundamentals::*;
pub use manager::*;
pub use socket::*;

#[cfg(test)]
pub mod tests {
    use file_system::AccessFlags;
    use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
    use virtual_file_system::{File, create_default_hierarchy};

    use super::*;

    extern crate abi_definitions;

    drivers_std::memory::instantiate_global_allocator!();

    pub(crate) async fn initialize() -> &'static crate::Manager {
        static INITIALIZE_MUTEX: Mutex<CriticalSectionRawMutex, bool> = Mutex::new(false);

        let mut initialized = INITIALIZE_MUTEX.lock().await;

        if *initialized {
            return crate::get_instance();
        }

        *initialized = true;

        static RANDOM_DEVICE: drivers_shared::devices::RandomDevice =
            drivers_shared::devices::RandomDevice;
        static TIME_DEVICE: drivers_std::devices::TimeDevice = drivers_std::devices::TimeDevice;

        let task_manager = task::initialize();
        let task = task_manager.get_current_task_identifier().await;

        log::initialize(&drivers_std::log::Logger).unwrap();

        let user_manager = users::initialize();

        let time_manager = time::initialize(&TIME_DEVICE).unwrap();

        let memory_device = file_system::MemoryDevice::<512>::new_static(10 * 1024 * 1024);

        let root_file_system = little_fs::FileSystem::get_or_format(memory_device, 512).unwrap();

        let virtual_file_system = virtual_file_system::initialize(
            task_manager,
            user_manager,
            time_manager,
            root_file_system,
        )
        .unwrap();

        create_default_hierarchy(virtual_file_system, task)
            .await
            .unwrap();

        let network_manager = crate::initialize(task_manager, virtual_file_system, &RANDOM_DEVICE);

        let (device, controler_device) = crate::create_loopback_device();

        let spawner = drivers_std::executor::new_thread_executor().await;

        network_manager
            .mount_interface(task, "loopback0", device, controler_device, Some(spawner))
            .await
            .unwrap();

        let mut file = File::open(
            virtual_file_system,
            task,
            "/devices/network/loopback0",
            AccessFlags::Write.into(),
        )
        .await
        .unwrap();

        file.control(ADD_IP_ADDRESS, &IpCidr::new_ipv4([127, 0, 0, 1], 8))
            .await
            .unwrap();

        file.close(virtual_file_system).await.unwrap();

        network_manager
    }
}
