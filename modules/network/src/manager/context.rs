use alloc::{boxed::Box, vec::Vec};
use embassy_net::{
    Stack,
    tcp::TcpSocket,
    udp::{PacketMetadata, UdpSocket},
};
use file_system::DirectCharacterDevice;

pub struct InterfaceContext<'a> {
    pub stack: Stack<'a>,
    pub controller: &'a dyn DirectCharacterDevice,
}

unsafe impl<'a> Send for InterfaceContext<'a> {}
unsafe impl<'a> Sync for InterfaceContext<'a> {}

pub struct TcpSocketContext<'a> {
    pub socket: TcpSocket<'a>,
    pub receive_buffer: Box<[u8]>,
    pub send_buffer: Box<[u8]>,
}

pub struct UdpSocketContext<'a> {
    pub socket: UdpSocket<'a>,
    pub transmit_meta_buffer: Box<[PacketMetadata]>,
    pub receive_meta_buffer: Box<[PacketMetadata]>,
    pub receive_buffer: Box<[u8]>,
    pub transmit_buffer: Box<[u8]>,
}
