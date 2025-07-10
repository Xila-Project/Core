use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream, UdpSocket},
    os::fd::{AsRawFd, FromRawFd, RawFd},
    sync::RwLock,
};

use core::mem::forget;

use file_system::{LocalFileIdentifier, LocalFileIdentifierIterator};
use network::{Error, IPv4, IPv6, Port, Protocol, Result, SocketDriver, IP};
use time::Duration;

use super::error::into_socket_error;

struct Inner {
    #[cfg(target_family = "unix")]
    pub sockets: BTreeMap<LocalFileIdentifier, RawFd>,
}

pub struct NetworkSocketDriver(RwLock<Inner>);

const fn into_socketaddr(ip: IP, port: Port) -> SocketAddr {
    let ip = match ip {
        IP::IPv4(ip) => {
            let ip = ip.into_inner();

            IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]))
        }

        IP::IPv6(ip) => {
            let ip = ip.into_inner();

            IpAddr::V6(Ipv6Addr::new(
                ip[0], ip[1], ip[2], ip[3], ip[4], ip[5], ip[6], ip[7],
            ))
        }
    };

    let port = port.into_inner();

    SocketAddr::new(ip, port)
}

const fn into_ip_and_port(socket_address: SocketAddr) -> (IP, Port) {
    let ip = match socket_address.ip() {
        IpAddr::V4(ip) => IP::IPv4(IPv4::new(ip.octets())),
        IpAddr::V6(ip) => IP::IPv6(IPv6::new(ip.segments())),
    };

    let port = Port::new(socket_address.port());

    (ip, port)
}

impl Default for NetworkSocketDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkSocketDriver {
    pub fn new() -> Self {
        Self(RwLock::new(Inner {
            sockets: BTreeMap::new(),
        }))
    }

    fn new_socket(&self, socket: LocalFileIdentifier, raw_socket: RawFd) -> Result<()> {
        let mut inner = self.0.write().unwrap();

        if inner.sockets.contains_key(&socket) {
            return Err(Error::DuplicateIdentifier);
        }

        if inner.sockets.insert(socket, raw_socket).is_some() {
            unreachable!();
        }

        Ok(())
    }

    fn get_socket(&self, socket: LocalFileIdentifier) -> Result<RawFd> {
        Ok(*self
            .0
            .read()
            .unwrap()
            .sockets
            .get(&socket)
            .ok_or(Error::InvalidIdentifier)?)
    }

    fn get_socket_mutable(&self, socket: LocalFileIdentifier) -> Result<RawFd> {
        Ok(*self
            .0
            .write()
            .unwrap()
            .sockets
            .get(&socket)
            .ok_or(Error::InvalidIdentifier)?)
    }

    fn remove_socket(&self, socket: LocalFileIdentifier) -> Result<RawFd> {
        self.0
            .write()
            .unwrap()
            .sockets
            .remove(&socket)
            .ok_or(Error::InvalidIdentifier)
    }
}

impl SocketDriver for NetworkSocketDriver {
    fn get_new_socket_identifier(
        &self,
        mut iterator: LocalFileIdentifierIterator,
    ) -> Result<Option<LocalFileIdentifier>> {
        let inner = self.0.read().unwrap();

        Ok(iterator.find(|identifier| !inner.sockets.contains_key(identifier)))
    }

    fn close(&self, socket: LocalFileIdentifier) -> Result<()> {
        let socket = self.remove_socket(socket)?;

        unsafe {
            let _ = TcpStream::from_raw_fd(socket);
        }

        Ok(())
    }

    fn bind(
        &self,
        ip: IP,
        port: Port,
        protocol: Protocol,
        socket: LocalFileIdentifier,
    ) -> Result<()> {
        match protocol {
            Protocol::TCP => {
                let tcp_listener =
                    TcpListener::bind(into_socketaddr(ip, port)).map_err(into_socket_error)?;

                self.new_socket(socket, tcp_listener.as_raw_fd())?;

                forget(tcp_listener); // * : Prevent closing the socket if the socket creation is SUCCESSFUL
            }

            Protocol::UDP => {
                let udp_socket =
                    UdpSocket::bind(into_socketaddr(ip, port)).map_err(into_socket_error)?;

                self.new_socket(socket, udp_socket.as_raw_fd())?;

                forget(udp_socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL
            }
            _ => return Err(Error::UnsupportedProtocol),
        };

        Ok(())
    }

    fn connect(&self, ip: IP, port: Port, socket: LocalFileIdentifier) -> Result<()> {
        let address = into_socketaddr(ip, port);

        let tcp_stream = TcpStream::connect(address).map_err(into_socket_error)?;

        self.new_socket(socket, tcp_stream.as_raw_fd())?;

        forget(tcp_stream); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn accept(
        &self,
        socket: LocalFileIdentifier,
        new_socket: LocalFileIdentifier,
    ) -> Result<(IP, Port)> {
        let socket = self.get_socket_mutable(socket)?;

        let tcp_listener = unsafe { TcpListener::from_raw_fd(socket) };

        let (tcp_stream, address) = tcp_listener.accept().map_err(into_socket_error)?;

        self.new_socket(new_socket, tcp_stream.as_raw_fd())?;

        forget(tcp_listener); // * : Prevent closing the socket if the socket creation is SUCCESSFUL
        forget(tcp_stream); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(into_ip_and_port(address))
    }

    fn send(&self, socket: LocalFileIdentifier, data: &[u8]) -> Result<()> {
        let socket = self.get_socket(socket)?;

        let mut socket = unsafe { TcpStream::from_raw_fd(socket) };

        socket.write_all(data).map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn receive(&self, socket: LocalFileIdentifier, data: &mut [u8]) -> Result<usize> {
        let socket = self.get_socket(socket)?;

        let mut socket = unsafe { TcpStream::from_raw_fd(socket) };

        let bytes = socket.read(data).map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(bytes)
    }

    fn receive_from(
        &self,
        socket: LocalFileIdentifier,
        data: &mut [u8],
    ) -> Result<(usize, IP, Port)> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { UdpSocket::from_raw_fd(socket) };

        let (bytes, address) = socket.recv_from(data).map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        let (ip, port) = into_ip_and_port(address);

        Ok((bytes, ip, port))
    }

    fn send_to(&self, socket: LocalFileIdentifier, data: &[u8], ip: IP, port: Port) -> Result<()> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { UdpSocket::from_raw_fd(socket) };

        let address = into_socketaddr(ip, port);

        socket.send_to(data, address).map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn get_local_address(&self, socket: LocalFileIdentifier) -> Result<(IP, Port)> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { TcpStream::from_raw_fd(socket) };

        let address = socket.local_addr().map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(into_ip_and_port(address))
    }

    fn get_remote_address(&self, socket: LocalFileIdentifier) -> Result<(IP, Port)> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { TcpStream::from_raw_fd(socket) };

        let address = socket.peer_addr().map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(into_ip_and_port(address))
    }

    fn set_send_timeout(&self, socket: LocalFileIdentifier, timeout: Duration) -> Result<()> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { TcpStream::from_raw_fd(socket) };

        socket
            .set_write_timeout(Some(timeout))
            .map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn set_receive_timeout(&self, socket: LocalFileIdentifier, timeout: Duration) -> Result<()> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { TcpStream::from_raw_fd(socket) };

        socket
            .set_read_timeout(Some(timeout))
            .map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn get_send_timeout(&self, socket: LocalFileIdentifier) -> Result<Option<Duration>> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { TcpStream::from_raw_fd(socket) };

        let timeout = socket.write_timeout().map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(timeout)
    }

    fn get_receive_timeout(&self, socket: LocalFileIdentifier) -> Result<Option<Duration>> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { TcpStream::from_raw_fd(socket) };

        let timeout = socket.read_timeout().map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(timeout)
    }
}

#[cfg(test)]
mod tests {
    use file_system::FileIdentifier;
    use task::TaskIdentifier;

    use super::*;
    use std::net::{TcpListener, UdpSocket};
    use std::os::fd::AsRawFd;

    pub const fn new_socket_identifier(identifier: FileIdentifier) -> LocalFileIdentifier {
        LocalFileIdentifier::new(TaskIdentifier::new(1), identifier)
    }

    #[test]
    fn test_new_socket() {
        let driver = NetworkSocketDriver::new();
        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let raw_fd = socket.as_raw_fd();

        let socket_identifier = new_socket_identifier(1.into());

        driver.new_socket(socket_identifier, raw_fd).unwrap();
        assert_eq!(driver.get_socket(socket_identifier).unwrap(), raw_fd);
    }

    #[test]
    fn test_remove_socket() {
        let driver = NetworkSocketDriver::new();
        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let raw_fd = socket.as_raw_fd();

        let socket_identifier = new_socket_identifier(1.into());

        driver.new_socket(socket_identifier, raw_fd).unwrap();
        assert_eq!(driver.remove_socket(socket_identifier).unwrap(), raw_fd);
        assert!(driver.get_socket(socket_identifier).is_err());
    }

    #[test]
    fn test_bind_tcp() {
        let driver = NetworkSocketDriver::new();

        let socket_identifier = new_socket_identifier(1.into());

        let ip = IP::IPv4(IPv4::new([127, 0, 0, 1]));
        let port = Port::new(0);

        driver
            .bind(ip, port, Protocol::TCP, socket_identifier)
            .unwrap();
    }

    #[test]
    fn test_bind_udp() {
        let driver = NetworkSocketDriver::new();

        let socket = new_socket_identifier(1.into());

        let ip = IP::IPv4(IPv4::new([127, 0, 0, 1]));
        let port = Port::new(0);

        driver.bind(ip, port, Protocol::UDP, socket).unwrap();
    }

    #[test]
    fn test_close() {
        let driver = NetworkSocketDriver::new();

        let socket_1 = new_socket_identifier(1.into());
        let socket_identifier_2 = new_socket_identifier(2.into());

        // - Bind sockets
        driver
            .bind(IPv4::LOCALHOST.into(), Port::ANY, Protocol::UDP, socket_1)
            .unwrap();

        driver
            .bind(
                IPv4::LOCALHOST.into(),
                Port::ANY,
                Protocol::UDP,
                socket_identifier_2,
            )
            .unwrap();

        // - Get local addresses
        let (ip_2, port_2) = driver.get_local_address(socket_identifier_2).unwrap();

        // - Send data from socket 1 to socket 2
        driver
            .send_to(socket_1, b"hello", ip_2.clone(), port_2)
            .unwrap();

        driver.close(socket_1).unwrap();

        assert_eq!(
            Error::InvalidIdentifier,
            driver
                .send_to(
                    socket_1,
                    b"hello",
                    IP::IPv4(IPv4::new([127, 0, 0, 1])),
                    Port::new(0),
                )
                .unwrap_err()
        );
    }

    #[test]
    fn test_connect() {
        let driver = NetworkSocketDriver::new();

        let socket_1 = new_socket_identifier(1.into());

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (ip, port) = into_ip_and_port(addr);

        driver.connect(ip, port, socket_1).unwrap();
    }

    #[test]
    fn test_send_receive() {
        let driver = NetworkSocketDriver::new();

        let socket_1_identifier = new_socket_identifier(1.into());

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (ip, port) = into_ip_and_port(addr);

        driver.connect(ip, port, socket_1_identifier).unwrap();
        let (mut stream, _) = listener.accept().unwrap();

        let data = b"hello";
        driver.send(socket_1_identifier, data).unwrap();

        let mut buffer = [0; 5];
        stream.read_exact(&mut buffer).unwrap();
        assert_eq!(&buffer, data);
    }

    #[test]
    fn test_tcp_send_receive_server() {
        let driver = NetworkSocketDriver::new();

        let server = new_socket_identifier(1.into());
        let server_stream = new_socket_identifier(2.into());

        // - Bind socket
        driver
            .bind(IPv4::LOCALHOST.into(), Port::ANY, Protocol::TCP, server)
            .unwrap();

        let (ip_server, port_server) = driver.get_local_address(server).unwrap();
        let server_address = into_socketaddr(ip_server.clone(), port_server);

        // - Connect to server
        let mut client = TcpStream::connect(server_address).unwrap();

        let (ip_client, port_client) = driver.accept(server, server_stream).unwrap();

        assert_eq!(
            driver.get_remote_address(server_stream).unwrap(),
            (ip_client, port_client)
        );
        assert_eq!(client.peer_addr().unwrap(), server_address);

        // - Send data from Client to Server
        let data = b"hello";

        client.write_all(data).unwrap();

        let mut buffer = [0; 5];
        let size = driver.receive(server_stream, &mut buffer).unwrap();

        assert_eq!(size, data.len());
        assert_eq!(&buffer, data);

        // - Send data from Server to Client
        let data = b"world";

        driver.send(server_stream, data).unwrap();

        let mut buffer = [0; 5];
        client.read_exact(&mut buffer).unwrap();

        assert_eq!(&buffer, data);

        // - Send data from Client to Server
        let data = b"fizzbuzz";

        client.write_all(data).unwrap();

        let mut buffer = [0; 8];
        let size = driver.receive(server_stream, &mut buffer).unwrap();

        assert_eq!(size, data.len());
        assert_eq!(&buffer, data);
    }

    #[test]
    fn test_tcp_send_receive_client() {
        let driver = NetworkSocketDriver::new();

        let client = new_socket_identifier(1.into());

        // - Bind socket
        let server_listener = TcpListener::bind("127.0.0.1:0").unwrap();

        let server_address = server_listener.local_addr().unwrap();
        let (ip_server, port_server) = into_ip_and_port(server_address);

        driver
            .connect(ip_server.clone(), port_server, client)
            .unwrap();

        let (mut server_stream, client_address) = server_listener.accept().unwrap();

        assert_eq!(
            driver.get_remote_address(client).unwrap(),
            (ip_server.clone(), port_server)
        );
        assert_eq!(
            driver.get_local_address(client).unwrap(),
            into_ip_and_port(client_address)
        );

        // - Send data from Client to Server
        let data = b"hello";

        server_stream.write_all(data).unwrap();

        let mut buffer = [0; 5];
        let size = driver.receive(client, &mut buffer).unwrap();

        assert_eq!(size, data.len());
        assert_eq!(&buffer, data);

        // - Send data from Server to Client
        let data = b"world";

        driver.send(client, data).unwrap();

        let mut buffer = [0; 5];
        server_stream.read_exact(&mut buffer).unwrap();

        assert_eq!(&buffer, data);

        // - Send data from Client to Server
        let data = b"fizzbuzz";

        server_stream.write_all(data).unwrap();

        let mut buffer = [0; 8];
        let size = driver.receive(client, &mut buffer).unwrap();

        assert_eq!(size, data.len());
        assert_eq!(&buffer, data);
    }

    #[test]
    fn test_tcp_send_receive_both_sides() {
        let driver = NetworkSocketDriver::new();

        let server_listener = new_socket_identifier(1.into());
        let server_stream = new_socket_identifier(2.into());
        let client = new_socket_identifier(3.into());

        // - Bind socket
        driver
            .bind(
                IPv4::LOCALHOST.into(),
                Port::ANY,
                Protocol::TCP,
                server_listener,
            )
            .unwrap();

        let (ip_server, port_server) = driver.get_local_address(server_listener).unwrap();

        // - Connect to server
        driver
            .connect(ip_server.clone(), port_server, client)
            .unwrap();

        let (ip_client, port_client) = driver.accept(server_listener, server_stream).unwrap();

        assert_eq!(
            driver.get_local_address(client).unwrap(),
            (ip_client.clone(), port_client)
        );
        assert_eq!(
            driver.get_remote_address(client).unwrap(),
            (ip_server.clone(), port_server)
        );

        // - Send data from Client to Server
        let data = b"hello";

        driver.send(client, data).unwrap();

        let mut buffer = [0; 5];
        let size = driver.receive(server_stream, &mut buffer).unwrap();

        assert_eq!(size, data.len());
        assert_eq!(&buffer, data);

        // - Send data from Server to Client
        let data = b"world";

        driver.send(server_stream, data).unwrap();

        let mut buffer = [0; 5];
        let size = driver.receive(client, &mut buffer).unwrap();

        assert_eq!(size, data.len());
        assert_eq!(&buffer, data);

        // - Send data from Client to Server
        let data = b"fizzbuzz";

        driver.send(client, data).unwrap();

        let mut buffer = [0; 8];
        let size = driver.receive(server_stream, &mut buffer).unwrap();

        assert_eq!(size, data.len());
        assert_eq!(&buffer, data);
    }

    #[test]
    fn test_udp_send_to_receive_from_one_side() {
        let driver = NetworkSocketDriver::new();

        let socket_1 = new_socket_identifier(1.into());

        // -  Bind sockets
        driver
            .bind(IPv4::LOCALHOST.into(), Port::ANY, Protocol::UDP, socket_1)
            .unwrap();

        let socket_2 = UdpSocket::bind("127.0.0.1:0").unwrap();

        // - Get local addresses
        let (ip_1, port_1) = driver.get_local_address(socket_1).unwrap();
        let socket_1_address = into_socketaddr(ip_1, port_1);
        let socket_2_address = socket_2.local_addr().unwrap();
        let (ip_2, port_2) = into_ip_and_port(socket_2_address);

        // - Send data from socket 1 to socket 2 (send)
        let data = b"world";
        driver
            .send_to(socket_1, data, ip_2.clone(), port_2)
            .unwrap();

        let mut buffer_2 = [0; 10];

        let (size, socket_address) = socket_2.recv_from(&mut buffer_2).unwrap();

        assert_eq!(size, data.len());
        assert_eq!(socket_address, socket_1_address);
        assert_eq!(&buffer_2[..size], data);

        // - Send data from socket 2 to socket 1 (receive)
        let data = b"hello";

        socket_2.send_to(data, socket_1_address).unwrap();

        let mut buffer = [0; 10];
        let (size, ip, port) = driver.receive_from(socket_1, &mut buffer).unwrap();

        assert_eq!(size, 5);
        assert_eq!((ip, port), (ip_2.clone(), port_2));
        assert_eq!(&buffer[..size], data);

        // - Send data from socket 1 to socket 2 (send)
        let data = b"fizzbuzz";
        driver
            .send_to(socket_1, data, ip_2.clone(), port_2)
            .unwrap();

        let mut buffer_2 = [0; 10];

        let (size, socket_address) = socket_2.recv_from(&mut buffer_2).unwrap();

        assert_eq!(size, data.len());
        assert_eq!(socket_address, socket_1_address);
        assert_eq!(&buffer_2[..size], data);

        driver.close(socket_1).unwrap();
    }

    #[test]
    fn test_udp_send_to_receive_from_both_sides() {
        let driver = NetworkSocketDriver::new();

        let socket_1_identifier = new_socket_identifier(1.into());
        let socket_2_identifier = new_socket_identifier(2.into());

        // - Bind sockets
        driver
            .bind(
                IPv4::LOCALHOST.into(),
                Port::ANY,
                Protocol::UDP,
                socket_1_identifier,
            )
            .unwrap();

        driver
            .bind(
                IPv4::LOCALHOST.into(),
                Port::ANY,
                Protocol::UDP,
                socket_2_identifier,
            )
            .unwrap();

        // - Get local addresses
        let (ip_1, port_1) = driver.get_local_address(socket_1_identifier).unwrap();
        let (ip_2, port_2) = driver.get_local_address(socket_2_identifier).unwrap();

        // - Send data from socket 1 to socket 2
        let data = b"hello";

        driver
            .send_to(socket_1_identifier, data, ip_2.clone(), port_2)
            .unwrap();

        let mut buffer = [0; 10];
        let (size, ip, port) = driver
            .receive_from(socket_2_identifier, &mut buffer)
            .unwrap();

        assert_eq!(size, data.len());
        assert_eq!((ip, port), (ip_1.clone(), port_1));
        assert_eq!(&buffer[..size], data);

        // - Send data from socket 2 to socket 1
        driver
            .send_to(socket_2_identifier, b"world", ip_1.clone(), port_1)
            .unwrap();

        let mut buffer = [0; 10];
        let (size, ip, port) = driver
            .receive_from(socket_1_identifier, &mut buffer)
            .unwrap();

        assert_eq!(size, data.len());
        assert_eq!((ip, port), (ip_2.clone(), port_2));
        assert_eq!(&buffer[..size], b"world");

        // - Send data from socket 1 to socket 2
        let data = b"fizzbuzz";

        driver
            .send_to(socket_1_identifier, data, ip_2.clone(), port_2)
            .unwrap();

        let mut buffer = [0; 10];
        let (size, ip, port) = driver
            .receive_from(socket_2_identifier, &mut buffer)
            .unwrap();

        assert_eq!(size, data.len());
        assert_eq!((ip, port), (ip_1, port_1));
        assert_eq!(&buffer[..size], data);

        driver.close(socket_1_identifier).unwrap();
    }
}
