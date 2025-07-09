use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream, UdpSocket},
    os::fd::{AsRawFd, FromRawFd, RawFd},
    sync::RwLock,
};

use core::mem::forget;

use file_system::{Local_file_identifier_iterator_type, Local_file_identifier_type};
use network::{
    Error_type, IP_type, IPv4_type, IPv6_type, Network_socket_driver_trait, Port_type,
    Protocol_type, Result_type,
};
use time::Duration_type;

use super::error::into_socket_error;

struct Inner_type {
    #[cfg(target_family = "unix")]
    pub sockets: BTreeMap<Local_file_identifier_type, RawFd>,
}

pub struct Network_socket_driver_type(RwLock<Inner_type>);

const fn into_socketaddr(ip: IP_type, port: Port_type) -> SocketAddr {
    let ip = match ip {
        IP_type::IPv4(ip) => {
            let ip = ip.Into_inner();

            IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]))
        }

        IP_type::IPv6(ip) => {
            let ip = ip.Into_inner();

            IpAddr::V6(Ipv6Addr::new(
                ip[0], ip[1], ip[2], ip[3], ip[4], ip[5], ip[6], ip[7],
            ))
        }
    };

    let port = port.Into_inner();

    SocketAddr::new(ip, port)
}

const fn into_ip_and_port(socket_address: SocketAddr) -> (IP_type, Port_type) {
    let ip = match socket_address.ip() {
        IpAddr::V4(ip) => IP_type::IPv4(IPv4_type::New(ip.octets())),
        IpAddr::V6(ip) => IP_type::IPv6(IPv6_type::new(ip.segments())),
    };

    let port = Port_type::New(socket_address.port());

    (ip, port)
}

impl Network_socket_driver_type {
    pub fn new() -> Self {
        Self(RwLock::new(Inner_type {
            sockets: BTreeMap::new(),
        }))
    }

    fn new_socket(&self, socket: Local_file_identifier_type, raw_socket: RawFd) -> Result_type<()> {
        let mut inner = self.0.write().unwrap();

        if inner.sockets.contains_key(&socket) {
            return Err(Error_type::Duplicate_identifier);
        }

        if inner.sockets.insert(socket, raw_socket).is_some() {
            unreachable!();
        }

        Ok(())
    }

    fn get_socket(&self, socket: Local_file_identifier_type) -> Result_type<RawFd> {
        Ok(*self
            .0
            .read()
            .unwrap()
            .sockets
            .get(&socket)
            .ok_or(Error_type::Invalid_identifier)?)
    }

    fn get_socket_mutable(&self, socket: Local_file_identifier_type) -> Result_type<RawFd> {
        Ok(*self
            .0
            .write()
            .unwrap()
            .sockets
            .get(&socket)
            .ok_or(Error_type::Invalid_identifier)?)
    }

    fn remove_socket(&self, socket: Local_file_identifier_type) -> Result_type<RawFd> {
        self.0
            .write()
            .unwrap()
            .sockets
            .remove(&socket)
            .ok_or(Error_type::Invalid_identifier)
    }
}

impl Network_socket_driver_trait for Network_socket_driver_type {
    fn get_new_socket_identifier(
        &self,
        mut iterator: Local_file_identifier_iterator_type,
    ) -> Result_type<Option<Local_file_identifier_type>> {
        let inner = self.0.read().unwrap();

        Ok(iterator.find(|identifier| !inner.sockets.contains_key(identifier)))
    }

    fn Close(&self, socket: Local_file_identifier_type) -> Result_type<()> {
        let socket = self.remove_socket(socket)?;

        unsafe {
            let _ = TcpStream::from_raw_fd(socket);
        }

        Ok(())
    }

    fn Bind(
        &self,
        ip: IP_type,
        port: Port_type,
        protocol: Protocol_type,
        socket: Local_file_identifier_type,
    ) -> Result_type<()> {
        match protocol {
            Protocol_type::TCP => {
                let tcp_listener =
                    TcpListener::bind(into_socketaddr(ip, port)).map_err(into_socket_error)?;

                self.new_socket(socket, tcp_listener.as_raw_fd())?;

                forget(tcp_listener); // * : Prevent closing the socket if the socket creation is SUCCESSFUL
            }

            Protocol_type::UDP => {
                let udp_socket =
                    UdpSocket::bind(into_socketaddr(ip, port)).map_err(into_socket_error)?;

                self.new_socket(socket, udp_socket.as_raw_fd())?;

                forget(udp_socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL
            }
            _ => return Err(Error_type::Unsupported_protocol),
        };

        Ok(())
    }

    fn Connect(
        &self,
        ip: IP_type,
        port: Port_type,
        socket: Local_file_identifier_type,
    ) -> Result_type<()> {
        let address = into_socketaddr(ip, port);

        let tcp_stream = TcpStream::connect(address).map_err(into_socket_error)?;

        self.new_socket(socket, tcp_stream.as_raw_fd())?;

        forget(tcp_stream); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn Accept(
        &self,
        socket: Local_file_identifier_type,
        new_socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)> {
        let socket = self.get_socket_mutable(socket)?;

        let tcp_listener = unsafe { TcpListener::from_raw_fd(socket) };

        let (tcp_stream, address) = tcp_listener.accept().map_err(into_socket_error)?;

        self.new_socket(new_socket, tcp_stream.as_raw_fd())?;

        forget(tcp_listener); // * : Prevent closing the socket if the socket creation is SUCCESSFUL
        forget(tcp_stream); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(into_ip_and_port(address))
    }

    fn Send(&self, socket: Local_file_identifier_type, data: &[u8]) -> Result_type<()> {
        let socket = self.get_socket(socket)?;

        let mut socket = unsafe { TcpStream::from_raw_fd(socket) };

        socket.write_all(data).map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn Receive(&self, socket: Local_file_identifier_type, data: &mut [u8]) -> Result_type<usize> {
        let socket = self.get_socket(socket)?;

        let mut socket = unsafe { TcpStream::from_raw_fd(socket) };

        let bytes = socket.read(data).map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(bytes)
    }

    fn Receive_from(
        &self,
        socket: Local_file_identifier_type,
        data: &mut [u8],
    ) -> Result_type<(usize, IP_type, Port_type)> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { UdpSocket::from_raw_fd(socket) };

        let (bytes, address) = socket.recv_from(data).map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        let (ip, port) = into_ip_and_port(address);

        Ok((bytes, ip, port))
    }

    fn Send_to(
        &self,
        socket: Local_file_identifier_type,
        data: &[u8],
        ip: IP_type,
        port: Port_type,
    ) -> Result_type<()> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { UdpSocket::from_raw_fd(socket) };

        let address = into_socketaddr(ip, port);

        socket.send_to(data, address).map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn get_local_address(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { TcpStream::from_raw_fd(socket) };

        let address = socket.local_addr().map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(into_ip_and_port(address))
    }

    fn get_remote_address(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { TcpStream::from_raw_fd(socket) };

        let address = socket.peer_addr().map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(into_ip_and_port(address))
    }

    fn Set_send_timeout(
        &self,
        socket: Local_file_identifier_type,
        timeout: Duration_type,
    ) -> Result_type<()> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { TcpStream::from_raw_fd(socket) };

        socket
            .set_write_timeout(Some(timeout))
            .map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn Set_receive_timeout(
        &self,
        socket: Local_file_identifier_type,
        timeout: Duration_type,
    ) -> Result_type<()> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { TcpStream::from_raw_fd(socket) };

        socket
            .set_read_timeout(Some(timeout))
            .map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn get_send_timeout(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<Option<Duration_type>> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { TcpStream::from_raw_fd(socket) };

        let timeout = socket.write_timeout().map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(timeout)
    }

    fn get_receive_timeout(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<Option<Duration_type>> {
        let socket = self.get_socket(socket)?;

        let socket = unsafe { TcpStream::from_raw_fd(socket) };

        let timeout = socket.read_timeout().map_err(into_socket_error)?;

        forget(socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(timeout)
    }
}

#[cfg(test)]
mod tests {
    use file_system::File_identifier_type;
    use task::Task_identifier_type;

    use super::*;
    use std::net::{TcpListener, UdpSocket};
    use std::os::fd::AsRawFd;

    pub const fn new_socket_identifier(
        identifier: File_identifier_type,
    ) -> Local_file_identifier_type {
        Local_file_identifier_type::New(Task_identifier_type::new(1), identifier)
    }

    #[test]
    fn test_new_socket() {
        let driver = Network_socket_driver_type::new();
        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let raw_fd = socket.as_raw_fd();

        let socket_identifier = new_socket_identifier(1.into());

        driver.new_socket(socket_identifier, raw_fd).unwrap();
        assert_eq!(driver.get_socket(socket_identifier).unwrap(), raw_fd);
    }

    #[test]
    fn test_remove_socket() {
        let driver = Network_socket_driver_type::new();
        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let raw_fd = socket.as_raw_fd();

        let socket_identifier = new_socket_identifier(1.into());

        driver.new_socket(socket_identifier, raw_fd).unwrap();
        assert_eq!(driver.remove_socket(socket_identifier).unwrap(), raw_fd);
        assert!(driver.get_socket(socket_identifier).is_err());
    }

    #[test]
    fn test_bind_tcp() {
        let driver = Network_socket_driver_type::new();

        let socket_identifier = new_socket_identifier(1.into());

        let ip = IP_type::IPv4(IPv4_type::New([127, 0, 0, 1]));
        let port = Port_type::New(0);

        driver
            .Bind(ip, port, Protocol_type::TCP, socket_identifier)
            .unwrap();
    }

    #[test]
    fn test_bind_udp() {
        let driver = Network_socket_driver_type::new();

        let socket = new_socket_identifier(1.into());

        let ip = IP_type::IPv4(IPv4_type::New([127, 0, 0, 1]));
        let port = Port_type::New(0);

        driver.Bind(ip, port, Protocol_type::UDP, socket).unwrap();
    }

    #[test]
    fn test_close() {
        let driver = Network_socket_driver_type::new();

        let socket_1 = new_socket_identifier(1.into());
        let socket_identifier_2 = new_socket_identifier(2.into());

        // - Bind sockets
        driver
            .Bind(
                IPv4_type::LOCALHOST.into(),
                Port_type::ANY,
                Protocol_type::UDP,
                socket_1,
            )
            .unwrap();

        driver
            .Bind(
                IPv4_type::LOCALHOST.into(),
                Port_type::ANY,
                Protocol_type::UDP,
                socket_identifier_2,
            )
            .unwrap();

        // - Get local addresses
        let (ip_2, port_2) = driver.get_local_address(socket_identifier_2).unwrap();

        // - Send data from socket 1 to socket 2
        driver
            .Send_to(socket_1, b"hello", ip_2.clone(), port_2)
            .unwrap();

        driver.Close(socket_1).unwrap();

        assert_eq!(
            Error_type::Invalid_identifier,
            driver
                .Send_to(
                    socket_1,
                    b"hello",
                    IP_type::IPv4(IPv4_type::New([127, 0, 0, 1])),
                    Port_type::New(0),
                )
                .unwrap_err()
        );
    }

    #[test]
    fn test_connect() {
        let driver = Network_socket_driver_type::new();

        let socket_1 = new_socket_identifier(1.into());

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (ip, port) = into_ip_and_port(addr);

        driver.Connect(ip, port, socket_1).unwrap();
    }

    #[test]
    fn test_send_receive() {
        let driver = Network_socket_driver_type::new();

        let socket_1_identifier = new_socket_identifier(1.into());

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (ip, port) = into_ip_and_port(addr);

        driver.Connect(ip, port, socket_1_identifier).unwrap();
        let (mut stream, _) = listener.accept().unwrap();

        let data = b"hello";
        driver.Send(socket_1_identifier, data).unwrap();

        let mut buffer = [0; 5];
        stream.read_exact(&mut buffer).unwrap();
        assert_eq!(&buffer, data);
    }

    #[test]
    fn test_tcp_send_receive_server() {
        let driver = Network_socket_driver_type::new();

        let server = new_socket_identifier(1.into());
        let server_stream = new_socket_identifier(2.into());

        // - Bind socket
        driver
            .Bind(
                IPv4_type::LOCALHOST.into(),
                Port_type::ANY,
                Protocol_type::TCP,
                server,
            )
            .unwrap();

        let (ip_server, port_server) = driver.get_local_address(server).unwrap();
        let server_address = into_socketaddr(ip_server.clone(), port_server);

        // - Connect to server
        let mut client = TcpStream::connect(server_address).unwrap();

        let (ip_client, port_client) = driver.Accept(server, server_stream).unwrap();

        assert_eq!(
            driver.get_remote_address(server_stream).unwrap(),
            (ip_client, port_client)
        );
        assert_eq!(client.peer_addr().unwrap(), server_address);

        // - Send data from Client to Server
        let data = b"hello";

        client.write_all(data).unwrap();

        let mut buffer = [0; 5];
        let size = driver.Receive(server_stream, &mut buffer).unwrap();

        assert_eq!(size, data.len());
        assert_eq!(&buffer, data);

        // - Send data from Server to Client
        let data = b"world";

        driver.Send(server_stream, data).unwrap();

        let mut buffer = [0; 5];
        client.read_exact(&mut buffer).unwrap();

        assert_eq!(&buffer, data);

        // - Send data from Client to Server
        let data = b"fizzbuzz";

        client.write_all(data).unwrap();

        let mut buffer = [0; 8];
        let size = driver.Receive(server_stream, &mut buffer).unwrap();

        assert_eq!(size, data.len());
        assert_eq!(&buffer, data);
    }

    #[test]
    fn test_tcp_send_receive_client() {
        let driver = Network_socket_driver_type::new();

        let client = new_socket_identifier(1.into());

        // - Bind socket
        let server_listener = TcpListener::bind("127.0.0.1:0").unwrap();

        let server_address = server_listener.local_addr().unwrap();
        let (ip_server, port_server) = into_ip_and_port(server_address);

        driver
            .Connect(ip_server.clone(), port_server, client)
            .unwrap();

        let (mut server_stream, Client_address) = server_listener.accept().unwrap();

        assert_eq!(
            driver.get_remote_address(client).unwrap(),
            (ip_server.clone(), port_server)
        );
        assert_eq!(
            driver.get_local_address(client).unwrap(),
            into_ip_and_port(Client_address)
        );

        // - Send data from Client to Server
        let data = b"hello";

        server_stream.write_all(data).unwrap();

        let mut buffer = [0; 5];
        let size = driver.Receive(client, &mut buffer).unwrap();

        assert_eq!(size, data.len());
        assert_eq!(&buffer, data);

        // - Send data from Server to Client
        let data = b"world";

        driver.Send(client, data).unwrap();

        let mut Buffer = [0; 5];
        server_stream.read_exact(&mut Buffer).unwrap();

        assert_eq!(&Buffer, data);

        // - Send data from Client to Server
        let data = b"fizzbuzz";

        server_stream.write_all(data).unwrap();

        let mut buffer = [0; 8];
        let Size = driver.Receive(client, &mut buffer).unwrap();

        assert_eq!(Size, data.len());
        assert_eq!(&buffer, data);
    }

    #[test]
    fn test_tcp_send_receive_both_sides() {
        let Driver = Network_socket_driver_type::new();

        let Server_listener = new_socket_identifier(1.into());
        let Server_stream = new_socket_identifier(2.into());
        let Client = new_socket_identifier(3.into());

        // - Bind socket
        Driver
            .Bind(
                IPv4_type::LOCALHOST.into(),
                Port_type::ANY,
                Protocol_type::TCP,
                Server_listener,
            )
            .unwrap();

        let (IP_server, Port_server) = Driver.get_local_address(Server_listener).unwrap();

        // - Connect to server
        Driver
            .Connect(IP_server.clone(), Port_server, Client)
            .unwrap();

        let (IP_client, Port_client) = Driver.Accept(Server_listener, Server_stream).unwrap();

        assert_eq!(
            Driver.get_local_address(Client).unwrap(),
            (IP_client.clone(), Port_client)
        );
        assert_eq!(
            Driver.get_remote_address(Client).unwrap(),
            (IP_server.clone(), Port_server)
        );

        // - Send data from Client to Server
        let Data = b"hello";

        Driver.Send(Client, Data).unwrap();

        let mut Buffer = [0; 5];
        let Size = Driver.Receive(Server_stream, &mut Buffer).unwrap();

        assert_eq!(Size, Data.len());
        assert_eq!(&Buffer, Data);

        // - Send data from Server to Client
        let Data = b"world";

        Driver.Send(Server_stream, Data).unwrap();

        let mut Buffer = [0; 5];
        let Size = Driver.Receive(Client, &mut Buffer).unwrap();

        assert_eq!(Size, Data.len());
        assert_eq!(&Buffer, Data);

        // - Send data from Client to Server
        let Data = b"fizzbuzz";

        Driver.Send(Client, Data).unwrap();

        let mut Buffer = [0; 8];
        let Size = Driver.Receive(Server_stream, &mut Buffer).unwrap();

        assert_eq!(Size, Data.len());
        assert_eq!(&Buffer, Data);
    }

    #[test]
    fn test_udp_send_to_receive_from_one_side() {
        let Driver = Network_socket_driver_type::new();

        let Socket_1 = new_socket_identifier(1.into());

        // -  Bind sockets
        Driver
            .Bind(
                IPv4_type::LOCALHOST.into(),
                Port_type::ANY,
                Protocol_type::UDP,
                Socket_1,
            )
            .unwrap();

        let Socket_2 = UdpSocket::bind("127.0.0.1:0").unwrap();

        // - Get local addresses
        let (IP_1, Port_1) = Driver.get_local_address(Socket_1).unwrap();
        let Socket_1_address = into_socketaddr(IP_1, Port_1);
        let Socket_2_address = Socket_2.local_addr().unwrap();
        let (IP_2, Port_2) = into_ip_and_port(Socket_2_address);

        // - Send data from socket 1 to socket 2 (send)
        let Data = b"world";
        Driver
            .Send_to(Socket_1, Data, IP_2.clone(), Port_2)
            .unwrap();

        let mut Buffer_2 = [0; 10];

        let (Size, Socket_address) = Socket_2.recv_from(&mut Buffer_2).unwrap();

        assert_eq!(Size, Data.len());
        assert_eq!(Socket_address, Socket_1_address);
        assert_eq!(&Buffer_2[..Size], Data);

        // - Send data from socket 2 to socket 1 (receive)
        let Data = b"hello";

        Socket_2.send_to(Data, Socket_1_address).unwrap();

        let mut Buffer = [0; 10];
        let (Size, IP, Port) = Driver.Receive_from(Socket_1, &mut Buffer).unwrap();

        assert_eq!(Size, 5);
        assert_eq!((IP, Port), (IP_2.clone(), Port_2));
        assert_eq!(&Buffer[..Size], Data);

        // - Send data from socket 1 to socket 2 (send)
        let Data = b"fizzbuzz";
        Driver
            .Send_to(Socket_1, Data, IP_2.clone(), Port_2)
            .unwrap();

        let mut Buffer_2 = [0; 10];

        let (Size, Socket_address) = Socket_2.recv_from(&mut Buffer_2).unwrap();

        assert_eq!(Size, Data.len());
        assert_eq!(Socket_address, Socket_1_address);
        assert_eq!(&Buffer_2[..Size], Data);

        Driver.Close(Socket_1).unwrap();
    }

    #[test]
    fn test_udp_send_to_receive_from_both_sides() {
        let Driver = Network_socket_driver_type::new();

        let Socket_1_identifier = new_socket_identifier(1.into());
        let Socket_2_identifier = new_socket_identifier(2.into());

        // - Bind sockets
        Driver
            .Bind(
                IPv4_type::LOCALHOST.into(),
                Port_type::ANY,
                Protocol_type::UDP,
                Socket_1_identifier,
            )
            .unwrap();

        Driver
            .Bind(
                IPv4_type::LOCALHOST.into(),
                Port_type::ANY,
                Protocol_type::UDP,
                Socket_2_identifier,
            )
            .unwrap();

        // - Get local addresses
        let (IP_1, Port_1) = Driver.get_local_address(Socket_1_identifier).unwrap();
        let (IP_2, Port_2) = Driver.get_local_address(Socket_2_identifier).unwrap();

        // - Send data from socket 1 to socket 2
        let Data = b"hello";

        Driver
            .Send_to(Socket_1_identifier, Data, IP_2.clone(), Port_2)
            .unwrap();

        let mut Buffer = [0; 10];
        let (Size, IP, Port) = Driver
            .Receive_from(Socket_2_identifier, &mut Buffer)
            .unwrap();

        assert_eq!(Size, Data.len());
        assert_eq!((IP, Port), (IP_1.clone(), Port_1));
        assert_eq!(&Buffer[..Size], Data);

        // - Send data from socket 2 to socket 1
        Driver
            .Send_to(Socket_2_identifier, b"world", IP_1.clone(), Port_1)
            .unwrap();

        let mut Buffer = [0; 10];
        let (Size, IP, Port) = Driver
            .Receive_from(Socket_1_identifier, &mut Buffer)
            .unwrap();

        assert_eq!(Size, Data.len());
        assert_eq!((IP, Port), (IP_2.clone(), Port_2));
        assert_eq!(&Buffer[..Size], b"world");

        // - Send data from socket 1 to socket 2
        let Data = b"fizzbuzz";

        Driver
            .Send_to(Socket_1_identifier, Data, IP_2.clone(), Port_2)
            .unwrap();

        let mut Buffer = [0; 10];
        let (Size, IP, Port) = Driver
            .Receive_from(Socket_2_identifier, &mut Buffer)
            .unwrap();

        assert_eq!(Size, Data.len());
        assert_eq!((IP, Port), (IP_1, Port_1));
        assert_eq!(&Buffer[..Size], Data);

        Driver.Close(Socket_1_identifier).unwrap();
    }
}
