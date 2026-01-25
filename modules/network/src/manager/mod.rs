mod context;
mod device;
mod runner;
mod stack;

use alloc::{vec, vec::Vec};
pub use context::*;
use file_system::{DirectCharacterDevice, Path};
pub use runner::*;
use smoltcp::{
    phy::Device,
    socket::{dns, icmp, tcp, udp},
};
use synchronization::once_lock::OnceLock;
use synchronization::{
    Arc, blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock, signal::Signal,
};
use task::{SpawnerIdentifier, TaskIdentifier};
use virtual_file_system::VirtualFileSystem;

use crate::{
    DnsSocket, Error, IcmpSocket, Result, TcpSocket, UdpSocket,
    manager::{
        device::NetworkDevice,
        stack::{Stack, StackInner},
    },
};

static MANAGER_INSTANCE: OnceLock<Manager> = OnceLock::new();

pub fn get_instance() -> &'static Manager {
    MANAGER_INSTANCE
        .try_get()
        .expect("Manager is not initialized")
}

pub fn initialize(
    _task_manager: &'static task::Manager,
    _virtual_file_system: &'static VirtualFileSystem,
    random_device: &'static dyn DirectCharacterDevice,
) -> &'static Manager {
    MANAGER_INSTANCE.get_or_init(|| Manager::new(random_device))
}

pub fn get_smoltcp_time() -> smoltcp::time::Instant {
    let time_manager = time::get_instance();
    let current_time = time_manager
        .get_current_time()
        .expect("Failed to get current time");

    smoltcp::time::Instant::from_millis(current_time.as_millis() as i64)
}

type StackList = Vec<Stack>;

pub struct Manager {
    pub(crate) random_device: &'static dyn DirectCharacterDevice,
    pub(crate) stacks: RwLock<CriticalSectionRawMutex, StackList>,
}

impl Manager {
    pub fn new(random_device: &'static dyn DirectCharacterDevice) -> Self {
        Manager {
            random_device,
            stacks: RwLock::new(Vec::new()),
        }
    }

    fn generate_seed(&self) -> Result<u64> {
        let mut buffer = [0u8; 8];
        self.random_device
            .read(&mut buffer, 0)
            .map_err(Error::FailedToGenerateSeed)?;
        Ok(u64::from_le_bytes(buffer))
    }

    pub(crate) async fn find_first_available_stack(stacks: &StackList) -> Option<Stack> {
        for stack in stacks {
            let available = stack.with(|s| s.is_available()).await;

            if available {
                return Some(stack.clone());
            }
        }

        None
    }

    pub(crate) async fn find_stack(stacks: &StackList, name: &str) -> Option<Stack> {
        for stack in stacks {
            let stack_name = stack.with(|s| s.name.clone()).await;

            if stack_name.as_str() == name {
                return Some(stack.clone());
            }
        }

        None
    }

    pub async fn mount_interface(
        &self,
        task: TaskIdentifier,
        name: &str,
        mut device: impl Device + 'static,
        controller_device: impl DirectCharacterDevice + 'static,
        spawner: Option<SpawnerIdentifier>,
    ) -> Result<()> {
        let mut stacks = self.stacks.write().await;

        if Self::find_stack(&stacks, name).await.is_some() {
            return Err(Error::DuplicateIdentifier);
        }

        let random_seed = self.generate_seed()?;
        let now = get_smoltcp_time();

        let stack_inner = StackInner::new(name, &mut device, controller_device, random_seed, now);

        // Create a wake signal for runner/stack communication
        let wake_signal: WakeSignal = Arc::new(Signal::new());

        let stack = Stack::new(stack_inner, wake_signal.clone());

        let mut runner = StackRunner::new(stack.clone(), device, wake_signal);

        let task_manager = task::get_instance();

        task_manager
            .spawn(
                task,
                "Network Interface Runner",
                spawner,
                move |_| async move {
                    runner.run().await;
                },
            )
            .await
            .map_err(Error::FailedToSpawnNetworkTask)?;

        let path = Path::NETWORK_DEVICES
            .join(Path::from_str(name))
            .ok_or(Error::InvalidIdentifier)?;

        let device = NetworkDevice::new(stack.clone());

        let virtual_file_system = virtual_file_system::get_instance();

        match virtual_file_system
            .create_directory(task, &Path::NETWORK_DEVICES)
            .await
        {
            Ok(_) => {}
            Err(virtual_file_system::Error::AlreadyExists) => {}
            Err(e) => return Err(Error::FailedToMountDevice(e)),
        };

        match virtual_file_system.remove(task, &path).await {
            Ok(_) => {}
            Err(virtual_file_system::Error::FileSystem(file_system::Error::NotFound)) => {}
            Err(e) => return Err(Error::FailedToMountDevice(e)),
        };

        virtual_file_system
            .mount_character_device(task, path, device)
            .await
            .map_err(Error::FailedToMountDevice)?;

        stacks.push(stack);

        Ok(())
    }

    pub async fn new_dns_socket(&self, interface_name: Option<&str>) -> Result<DnsSocket> {
        let stacks = self.stacks.read().await;

        let stack = if let Some(name) = interface_name {
            Self::find_stack(&stacks, name)
                .await
                .ok_or(Error::NotFound)?
        } else {
            Self::find_first_available_stack(&stacks)
                .await
                .ok_or(Error::NotFound)?
        };

        let handle = stack
            .with_mutable(|s| {
                let socket = dns::Socket::new(&s.dns_servers, vec![]);
                s.add_socket(socket)
            })
            .await;

        let context = SocketContext {
            handle,
            stack: stack.clone(),
            closed: false,
        };
        let socket = DnsSocket::new(context);

        Ok(socket)
    }

    pub async fn new_tcp_socket(
        &self,
        transmit_buffer_size: usize,
        receive_buffer_size: usize,
        interface_name: Option<&str>,
    ) -> Result<TcpSocket> {
        let stacks = self.stacks.read().await;

        let stack = if let Some(name) = interface_name {
            Self::find_stack(&stacks, name)
                .await
                .ok_or(Error::NotFound)?
        } else {
            Self::find_first_available_stack(&stacks)
                .await
                .ok_or(Error::NotFound)?
        };

        let send_buffer = tcp::SocketBuffer::new(vec![0u8; transmit_buffer_size]);
        let receive_buffer = tcp::SocketBuffer::new(vec![0u8; receive_buffer_size]);

        let socket = tcp::Socket::new(receive_buffer, send_buffer);
        let handle = stack.with_mutable(|s| s.add_socket(socket)).await;

        let context = SocketContext {
            handle,
            stack: stack.clone(),
            closed: false,
        };

        Ok(TcpSocket::new(context))
    }

    pub async fn new_udp_socket(
        &self,
        transmit_buffer_size: usize,
        receive_buffer_size: usize,
        receive_meta_buffer_size: usize,
        transmit_meta_buffer_size: usize,
        interface_name: Option<&str>,
    ) -> Result<UdpSocket> {
        let stacks = self.stacks.read().await;

        let stack = if let Some(name) = interface_name {
            Self::find_stack(&stacks, name)
                .await
                .ok_or(Error::NotFound)?
        } else {
            Self::find_first_available_stack(&stacks)
                .await
                .ok_or(Error::NotFound)?
        };

        let receive_meta_buffer = udp::PacketBuffer::new(
            vec![udp::PacketMetadata::EMPTY; receive_meta_buffer_size],
            vec![0u8; receive_buffer_size],
        );
        let transmit_meta_buffer = udp::PacketBuffer::new(
            vec![udp::PacketMetadata::EMPTY; transmit_meta_buffer_size],
            vec![0u8; transmit_buffer_size],
        );

        let socket = udp::Socket::new(receive_meta_buffer, transmit_meta_buffer);
        let handle = stack.with_mutable(|s| s.add_socket(socket)).await;

        let context = SocketContext {
            handle,
            stack: stack.clone(),
            closed: false,
        };

        Ok(UdpSocket::new(context))
    }

    pub async fn new_icmp_socket(
        &self,
        receive_buffer_size: usize,
        transmit_buffer_size: usize,
        receive_meta_buffer_size: usize,
        transmit_meta_buffer_size: usize,
        interface_name: Option<&str>,
    ) -> Result<IcmpSocket> {
        let stacks = self.stacks.read().await;

        let stack = if let Some(name) = interface_name {
            Self::find_stack(&stacks, name)
                .await
                .ok_or(Error::NotFound)?
        } else {
            Self::find_first_available_stack(&stacks)
                .await
                .ok_or(Error::NotFound)?
        };

        let receive_buffer = icmp::PacketBuffer::new(
            vec![icmp::PacketMetadata::EMPTY; receive_meta_buffer_size],
            vec![0u8; receive_buffer_size],
        );
        let transmit_buffer = icmp::PacketBuffer::new(
            vec![icmp::PacketMetadata::EMPTY; transmit_meta_buffer_size],
            vec![0u8; transmit_buffer_size],
        );

        let socket = icmp::Socket::new(receive_buffer, transmit_buffer);
        let handle = stack.with_mutable(|s| s.add_socket(socket)).await;

        let context = SocketContext {
            handle,
            stack: stack.clone(),
            closed: false,
        };

        let socket = IcmpSocket::new(context);

        Ok(socket)
    }
}
