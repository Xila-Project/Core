use core::mem::transmute;

use alloc::{boxed::Box, vec, vec::Vec};
use embassy_net::{
    Config, Stack, StackResources,
    tcp::TcpSocket,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_net_driver::Driver;
use file_system::DirectCharacterDevice;
use smol_str::SmolStr;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use task::TaskIdentifier;

use crate::{DNSQueryKind, Error, IP, Result, TcpSocketContext, UdpSocketContext};

pub(crate) struct Interface<'a> {
    pub name: SmolStr,
    pub stack: Stack<'a>,
    pub controller: &'a dyn DirectCharacterDevice,
}

unsafe impl Send for Interface<'_> {}
unsafe impl Sync for Interface<'_> {}

type InterfacesList<'a> = Vec<Interface<'a>>;

pub struct Manager<'a> {
    pub(crate) random_device: &'static dyn DirectCharacterDevice,
    pub(crate) interfaces: RwLock<CriticalSectionRawMutex, InterfacesList<'a>>,
}

impl<'a> Manager<'a> {
    pub fn new(random_device: &'static dyn DirectCharacterDevice) -> Self {
        Manager {
            random_device,
            interfaces: RwLock::new(Vec::new()),
        }
    }

    fn generate_seed(&self) -> Result<u64> {
        let mut buffer = [0u8; 8];
        self.random_device
            .read(&mut buffer, 0)
            .map_err(Error::FailedToGenerateSeed)?;
        Ok(u64::from_le_bytes(buffer))
    }

    pub(crate) fn find_first_up_interface(interfaces: &InterfacesList<'_>) -> Option<usize> {
        interfaces
            .iter()
            .position(|interface| interface.stack.is_link_up())
    }

    pub(crate) fn find_interface(interfaces: &InterfacesList<'_>, name: &str) -> Option<usize> {
        interfaces
            .iter()
            .position(|interface| interface.name == name)
    }

    pub async fn mount_interface(
        &self,
        task: TaskIdentifier,
        name: &str,
        driver: impl Driver + 'static,
        controller: impl DirectCharacterDevice + 'static,
    ) -> Result<()> {
        let mut interfaces = self.interfaces.write().await;

        if Self::find_interface(&interfaces, name).is_some() {
            return Err(Error::DuplicateIdentifier);
        }

        let configuration = Config::default();

        let ressources = Box::leak(Box::new(StackResources::<10>::new()));

        let random_seed = self.generate_seed()?;

        let (stack, mut runner) = embassy_net::new(driver, configuration, ressources, random_seed);

        let task_manager = task::get_instance();

        task_manager
            .spawn(
                task,
                "Network Interface Runner",
                None,
                move |_| async move {
                    runner.run().await;
                },
            )
            .await
            .map_err(Error::FailedToSpawnNetworkTask)?;

        let interface = Interface {
            name: name.into(),
            stack,
            controller: Box::leak(Box::new(controller)),
        };

        interfaces.push(interface);

        Ok(())
    }

    pub async fn resolve(
        &self,
        host: &str,
        kind: DNSQueryKind,
        interface_name: Option<&str>,
    ) -> Result<Vec<IP>> {
        let interfaces = self.interfaces.read().await;

        let interfaces = if let Some(name) = interface_name {
            let interface_index = Self::find_interface(&interfaces, name).ok_or(Error::Failed)?;

            interfaces[interface_index..=interface_index].iter()
        } else {
            interfaces.iter()
        };

        for interface in interfaces {
            if !interface.stack.is_link_up() {
                continue;
            }

            if let Ok(ip_address) = interface.stack.dns_query(host, kind).await {
                return Ok(ip_address
                    .into_iter()
                    .map(|addr| IP::from_embassy_address(addr))
                    .collect());
            }
        }

        Err(Error::Failed)
    }

    pub async fn new_tcp_socket(
        &self,
        transmit_buffer_size: usize,
        receive_buffer_size: usize,
        interface_name: Option<&str>,
    ) -> Result<TcpSocketContext<'a>> {
        let interfaces = self.interfaces.read().await;

        let interface_index = if let Some(name) = interface_name {
            Self::find_interface(&interfaces, name).ok_or(Error::NotFound)?
        } else {
            Self::find_first_up_interface(&interfaces).ok_or(Error::NotFound)?
        };

        let interface = &interfaces[interface_index];

        let mut send_buffer = vec![0u8; transmit_buffer_size].into_boxed_slice();
        let mut receive_buffer = vec![0u8; receive_buffer_size].into_boxed_slice();

        let send_slice = &mut send_buffer[..];
        let receive_slice = &mut receive_buffer[..];

        let transmit_slice = unsafe { transmute::<&mut [u8], &'a mut [u8]>(send_slice) };
        let receive_slice = unsafe { transmute::<&mut [u8], &'a mut [u8]>(receive_slice) };

        let listener = TcpSocket::new(interface.stack, transmit_slice, receive_slice);

        let context = TcpSocketContext {
            socket: listener,
            receive_buffer,
            send_buffer,
        };

        Ok(context)
    }

    pub async fn new_udp_socket(
        &self,
        transmit_buffer_size: usize,
        receive_buffer_size: usize,
        interface_name: Option<&str>,
    ) -> Result<UdpSocketContext<'a>> {
        let interfaces = self.interfaces.read().await;

        let interface_index = if let Some(name) = interface_name {
            Self::find_interface(&interfaces, name).ok_or(Error::NotFound)?
        } else {
            Self::find_first_up_interface(&interfaces).ok_or(Error::NotFound)?
        };

        let mut receive_meta_buffer =
            vec![PacketMetadata::EMPTY; receive_buffer_size].into_boxed_slice();
        let mut transmit_meta_buffer =
            vec![PacketMetadata::EMPTY; transmit_buffer_size].into_boxed_slice();
        let mut transmit_buffer = vec![0u8; transmit_buffer_size].into_boxed_slice();
        let mut receive_buffer = vec![0u8; receive_buffer_size].into_boxed_slice();

        let receive_meta_slice = &mut receive_meta_buffer[..];
        let transmit_meta_slice = &mut transmit_meta_buffer[..];
        let transmit_slice = &mut transmit_buffer[..];
        let receive_slice = &mut receive_buffer[..];

        let receive_meta_slice = unsafe {
            transmute::<&mut [PacketMetadata], &'a mut [PacketMetadata]>(receive_meta_slice)
        };
        let transmit_meta_slice = unsafe {
            transmute::<&mut [PacketMetadata], &'a mut [PacketMetadata]>(transmit_meta_slice)
        };
        let transmit_slice = unsafe { transmute::<&mut [u8], &'a mut [u8]>(transmit_slice) };
        let receive_slice = unsafe { transmute::<&mut [u8], &'a mut [u8]>(receive_slice) };

        let interface = &interfaces[interface_index];

        let socket = UdpSocket::new(
            interface.stack,
            receive_meta_slice,
            receive_slice,
            transmit_meta_slice,
            transmit_slice,
        );

        let socket = UdpSocketContext {
            socket,
            receive_meta_buffer,
            transmit_meta_buffer,
            receive_buffer,
            transmit_buffer,
        };

        Ok(socket)
    }
}
