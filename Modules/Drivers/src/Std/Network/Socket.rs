use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream, UdpSocket},
    os::fd::{AsRawFd, FromRawFd, RawFd},
    sync::RwLock,
};

use core::mem::forget;

use File_system::{Local_file_identifier_iterator_type, Local_file_identifier_type};
use Network::{
    Error_type, IP_type, IPv4_type, IPv6_type, Network_socket_driver_trait, Port_type,
    Protocol_type, Result_type,
};
use Time::Duration_type;

use crate::Std::Network::Error::Into_socket_error;

struct Inner_type {
    #[cfg(target_family = "unix")]
    pub sockets: BTreeMap<Local_file_identifier_type, RawFd>,
}

pub struct Network_socket_driver_type(RwLock<Inner_type>);

const fn Into_socketaddr(IP: IP_type, Port: Port_type) -> SocketAddr {
    let ip = match IP {
        IP_type::IPv4(ip) => {
            let ip = ip.Into_inner();

            IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]))
        }

        IP_type::IPv6(IP) => {
            let ip = IP.Into_inner();

            IpAddr::V6(Ipv6Addr::new(
                ip[0], ip[1], ip[2], ip[3], ip[4], ip[5], ip[6], ip[7],
            ))
        }
    };

    let Port = Port.Into_inner();

    SocketAddr::new(ip, Port)
}

const fn Into_IP_and_port(Socket_address: SocketAddr) -> (IP_type, Port_type) {
    let ip = match Socket_address.ip() {
        IpAddr::V4(ip) => IP_type::IPv4(IPv4_type::New(ip.octets())),
        IpAddr::V6(ip) => IP_type::IPv6(IPv6_type::new(ip.segments())),
    };

    let Port = Port_type::New(Socket_address.port());

    (ip, Port)
}

impl Network_socket_driver_type {
    pub fn new() -> Self {
        Self(RwLock::new(Inner_type {
            sockets: BTreeMap::new(),
        }))
    }

    fn New_socket(&self, Socket: Local_file_identifier_type, Raw_socket: RawFd) -> Result_type<()> {
        let mut inner = self.0.write().unwrap();

        if inner.sockets.contains_key(&Socket) {
            return Err(Error_type::Duplicate_identifier);
        }

        if inner.sockets.insert(Socket, Raw_socket).is_some() {
            unreachable!();
        }

        Ok(())
    }

    fn get_socket(&self, Socket: Local_file_identifier_type) -> Result_type<RawFd> {
        Ok(*self
            .0
            .read()
            .unwrap()
            .sockets
            .get(&Socket)
            .ok_or(Error_type::Invalid_identifier)?)
    }

    fn get_socket_mutable(&self, Socket: Local_file_identifier_type) -> Result_type<RawFd> {
        Ok(*self
            .0
            .write()
            .unwrap()
            .sockets
            .get(&Socket)
            .ok_or(Error_type::Invalid_identifier)?)
    }

    fn Remove_socket(&self, Socket: Local_file_identifier_type) -> Result_type<RawFd> {
        self.0
            .write()
            .unwrap()
            .sockets
            .remove(&Socket)
            .ok_or(Error_type::Invalid_identifier)
    }
}

impl Network_socket_driver_trait for Network_socket_driver_type {
    fn get_new_socket_identifier(
        &self,
        mut iterator: Local_file_identifier_iterator_type,
    ) -> Result_type<Option<Local_file_identifier_type>> {
        let inner = self.0.read().unwrap();

        Ok(iterator.find(|Identifier| !inner.sockets.contains_key(Identifier)))
    }

    fn Close(&self, Socket: Local_file_identifier_type) -> Result_type<()> {
        let socket = self.Remove_socket(Socket)?;

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
                    TcpListener::bind(Into_socketaddr(ip, port)).map_err(Into_socket_error)?;

                self.New_socket(socket, tcp_listener.as_raw_fd())?;

                forget(tcp_listener); // * : Prevent closing the socket if the socket creation is SUCCESSFUL
            }

            Protocol_type::UDP => {
                let udp_socket =
                    UdpSocket::bind(Into_socketaddr(ip, port)).map_err(Into_socket_error)?;

                self.New_socket(socket, udp_socket.as_raw_fd())?;

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
        let address = Into_socketaddr(ip, port);

        let TCP_stream = TcpStream::connect(address).map_err(Into_socket_error)?;

        self.New_socket(socket, TCP_stream.as_raw_fd())?;

        forget(TCP_stream); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn Accept(
        &self,
        socket: Local_file_identifier_type,
        new_socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)> {
        let socket = self.get_socket_mutable(socket)?;

        let TCP_listener = unsafe { TcpListener::from_raw_fd(socket) };

        let (TCP_stream, Address) = TCP_listener.accept().map_err(Into_socket_error)?;

        self.New_socket(new_socket, TCP_stream.as_raw_fd())?;

        forget(TCP_listener); // * : Prevent closing the socket if the socket creation is SUCCESSFUL
        forget(TCP_stream); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(Into_IP_and_port(Address))
    }

    fn Send(&self, Socket: Local_file_identifier_type, Data: &[u8]) -> Result_type<()> {
        let socket = self.get_socket(Socket)?;

        let mut Socket = unsafe { TcpStream::from_raw_fd(socket) };

        Socket.write_all(Data).map_err(Into_socket_error)?;

        forget(Socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn Receive(&self, Socket: Local_file_identifier_type, Data: &mut [u8]) -> Result_type<usize> {
        let socket = self.get_socket(Socket)?;

        let mut Socket = unsafe { TcpStream::from_raw_fd(socket) };

        let Bytes = Socket.read(Data).map_err(Into_socket_error)?;

        forget(Socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(Bytes)
    }

    fn Receive_from(
        &self,
        socket: Local_file_identifier_type,
        data: &mut [u8],
    ) -> Result_type<(usize, IP_type, Port_type)> {
        let socket = self.get_socket(socket)?;

        let Socket = unsafe { UdpSocket::from_raw_fd(socket) };

        let (Bytes, Address) = Socket.recv_from(data).map_err(Into_socket_error)?;

        forget(Socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        let (IP, Port) = Into_IP_and_port(Address);

        Ok((Bytes, IP, Port))
    }

    fn Send_to(
        &self,
        socket: Local_file_identifier_type,
        data: &[u8],
        ip: IP_type,
        port: Port_type,
    ) -> Result_type<()> {
        let socket = self.get_socket(socket)?;

        let Socket = unsafe { UdpSocket::from_raw_fd(socket) };

        let Address = Into_socketaddr(ip, port);

        Socket.send_to(data, Address).map_err(Into_socket_error)?;

        forget(Socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn get_local_address(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)> {
        let socket = self.get_socket(socket)?;

        let Socket = unsafe { TcpStream::from_raw_fd(socket) };

        let Address = Socket.local_addr().map_err(Into_socket_error)?;

        forget(Socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(Into_IP_and_port(Address))
    }

    fn get_remote_address(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<(IP_type, Port_type)> {
        let socket = self.get_socket(socket)?;

        let Socket = unsafe { TcpStream::from_raw_fd(socket) };

        let Address = Socket.peer_addr().map_err(Into_socket_error)?;

        forget(Socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(Into_IP_and_port(Address))
    }

    fn Set_send_timeout(
        &self,
        socket: Local_file_identifier_type,
        timeout: Duration_type,
    ) -> Result_type<()> {
        let socket = self.get_socket(socket)?;

        let Socket = unsafe { TcpStream::from_raw_fd(socket) };

        Socket
            .set_write_timeout(Some(timeout))
            .map_err(Into_socket_error)?;

        forget(Socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn Set_receive_timeout(
        &self,
        socket: Local_file_identifier_type,
        timeout: Duration_type,
    ) -> Result_type<()> {
        let socket = self.get_socket(socket)?;

        let Socket = unsafe { TcpStream::from_raw_fd(socket) };

        Socket
            .set_read_timeout(Some(timeout))
            .map_err(Into_socket_error)?;

        forget(Socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(())
    }

    fn get_send_timeout(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<Option<Duration_type>> {
        let socket = self.get_socket(socket)?;

        let Socket = unsafe { TcpStream::from_raw_fd(socket) };

        let Timeout = Socket.write_timeout().map_err(Into_socket_error)?;

        forget(Socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(Timeout)
    }

    fn get_receive_timeout(
        &self,
        socket: Local_file_identifier_type,
    ) -> Result_type<Option<Duration_type>> {
        let socket = self.get_socket(socket)?;

        let Socket = unsafe { TcpStream::from_raw_fd(socket) };

        let Timeout = Socket.read_timeout().map_err(Into_socket_error)?;

        forget(Socket); // * : Prevent closing the socket if the socket creation is SUCCESSFUL

        Ok(Timeout)
    }
}

#[cfg(test)]
mod tests {
    use File_system::File_identifier_type;
    use Task::Task_identifier_type;

    use super::*;
    use std::net::{TcpListener, UdpSocket};
    use std::os::fd::AsRawFd;

    pub const fn New_socket_identifier(
        Identifier: File_identifier_type,
    ) -> Local_file_identifier_type {
        Local_file_identifier_type::New(Task_identifier_type::new(1), Identifier)
    }

    #[test]
    fn test_new_socket() {
        let Driver = Network_socket_driver_type::new();
        let Socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let Raw_fd = Socket.as_raw_fd();

        let Socket_identifier = New_socket_identifier(1.into());

        Driver.New_socket(Socket_identifier, Raw_fd).unwrap();
        assert_eq!(Driver.get_socket(Socket_identifier).unwrap(), Raw_fd);
    }

    #[test]
    fn test_remove_socket() {
        let Driver = Network_socket_driver_type::new();
        let Socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let Raw_fd = Socket.as_raw_fd();

        let Socket_identifier = New_socket_identifier(1.into());

        Driver.New_socket(Socket_identifier, Raw_fd).unwrap();
        assert_eq!(Driver.Remove_socket(Socket_identifier).unwrap(), Raw_fd);
        assert!(Driver.get_socket(Socket_identifier).is_err());
    }

    #[test]
    fn test_bind_tcp() {
        let Driver = Network_socket_driver_type::new();

        let Socket_identifier = New_socket_identifier(1.into());

        let IP = IP_type::IPv4(IPv4_type::New([127, 0, 0, 1]));
        let Port = Port_type::New(0);

        Driver
            .Bind(IP, Port, Protocol_type::TCP, Socket_identifier)
            .unwrap();
    }

    #[test]
    fn test_bind_udp() {
        let Driver = Network_socket_driver_type::new();

        let Socket = New_socket_identifier(1.into());

        let IP = IP_type::IPv4(IPv4_type::New([127, 0, 0, 1]));
        let Port = Port_type::New(0);

        Driver.Bind(IP, Port, Protocol_type::UDP, Socket).unwrap();
    }

    #[test]
    fn test_close() {
        let Driver = Network_socket_driver_type::new();

        let Socket_1 = New_socket_identifier(1.into());
        let Socket_identifier_2 = New_socket_identifier(2.into());

        // - Bind sockets
        Driver
            .Bind(
                IPv4_type::LOCALHOST.into(),
                Port_type::ANY,
                Protocol_type::UDP,
                Socket_1,
            )
            .unwrap();

        Driver
            .Bind(
                IPv4_type::LOCALHOST.into(),
                Port_type::ANY,
                Protocol_type::UDP,
                Socket_identifier_2,
            )
            .unwrap();

        // - Get local addresses
        let (IP_2, Port_2) = Driver.get_local_address(Socket_identifier_2).unwrap();

        // - Send data from socket 1 to socket 2
        Driver
            .Send_to(Socket_1, b"hello", IP_2.clone(), Port_2)
            .unwrap();

        Driver.Close(Socket_1).unwrap();

        assert_eq!(
            Error_type::Invalid_identifier,
            Driver
                .Send_to(
                    Socket_1,
                    b"hello",
                    IP_type::IPv4(IPv4_type::New([127, 0, 0, 1])),
                    Port_type::New(0),
                )
                .unwrap_err()
        );
    }

    #[test]
    fn test_connect() {
        let Driver = Network_socket_driver_type::new();

        let Socket_1 = New_socket_identifier(1.into());

        let Listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let Addr = Listener.local_addr().unwrap();
        let (IP, Port) = Into_IP_and_port(Addr);

        Driver.Connect(IP, Port, Socket_1).unwrap();
    }

    #[test]
    fn test_send_receive() {
        let Driver = Network_socket_driver_type::new();

        let Socket_1_identifier = New_socket_identifier(1.into());

        let Listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let Addr = Listener.local_addr().unwrap();
        let (IP, Port) = Into_IP_and_port(Addr);

        Driver.Connect(IP, Port, Socket_1_identifier).unwrap();
        let (mut Stream, _) = Listener.accept().unwrap();

        let Data = b"hello";
        Driver.Send(Socket_1_identifier, Data).unwrap();

        let mut Buffer = [0; 5];
        Stream.read_exact(&mut Buffer).unwrap();
        assert_eq!(&Buffer, Data);
    }

    #[test]
    fn test_tcp_send_receive_server() {
        let Driver = Network_socket_driver_type::new();

        let Server = New_socket_identifier(1.into());
        let Server_stream = New_socket_identifier(2.into());

        // - Bind socket
        Driver
            .Bind(
                IPv4_type::LOCALHOST.into(),
                Port_type::ANY,
                Protocol_type::TCP,
                Server,
            )
            .unwrap();

        let (IP_server, Port_server) = Driver.get_local_address(Server).unwrap();
        let Server_address = Into_socketaddr(IP_server.clone(), Port_server);

        // - Connect to server
        let mut Client = TcpStream::connect(Server_address).unwrap();

        let (IP_client, Port_client) = Driver.Accept(Server, Server_stream).unwrap();

        assert_eq!(
            Driver.get_remote_address(Server_stream).unwrap(),
            (IP_client, Port_client)
        );
        assert_eq!(Client.peer_addr().unwrap(), Server_address);

        // - Send data from Client to Server
        let Data = b"hello";

        Client.write_all(Data).unwrap();

        let mut Buffer = [0; 5];
        let Size = Driver.Receive(Server_stream, &mut Buffer).unwrap();

        assert_eq!(Size, Data.len());
        assert_eq!(&Buffer, Data);

        // - Send data from Server to Client
        let Data = b"world";

        Driver.Send(Server_stream, Data).unwrap();

        let mut Buffer = [0; 5];
        Client.read_exact(&mut Buffer).unwrap();

        assert_eq!(&Buffer, Data);

        // - Send data from Client to Server
        let Data = b"fizzbuzz";

        Client.write_all(Data).unwrap();

        let mut Buffer = [0; 8];
        let Size = Driver.Receive(Server_stream, &mut Buffer).unwrap();

        assert_eq!(Size, Data.len());
        assert_eq!(&Buffer, Data);
    }

    #[test]
    fn test_TCP_send_receive_client() {
        let Driver = Network_socket_driver_type::new();

        let Client = New_socket_identifier(1.into());

        // - Bind socket
        let Server_listener = TcpListener::bind("127.0.0.1:0").unwrap();

        let Server_address = Server_listener.local_addr().unwrap();
        let (IP_server, Port_server) = Into_IP_and_port(Server_address);

        Driver
            .Connect(IP_server.clone(), Port_server, Client)
            .unwrap();

        let (mut Server_stream, Client_address) = Server_listener.accept().unwrap();

        assert_eq!(
            Driver.get_remote_address(Client).unwrap(),
            (IP_server.clone(), Port_server)
        );
        assert_eq!(
            Driver.get_local_address(Client).unwrap(),
            Into_IP_and_port(Client_address)
        );

        // - Send data from Client to Server
        let Data = b"hello";

        Server_stream.write_all(Data).unwrap();

        let mut Buffer = [0; 5];
        let Size = Driver.Receive(Client, &mut Buffer).unwrap();

        assert_eq!(Size, Data.len());
        assert_eq!(&Buffer, Data);

        // - Send data from Server to Client
        let Data = b"world";

        Driver.Send(Client, Data).unwrap();

        let mut Buffer = [0; 5];
        Server_stream.read_exact(&mut Buffer).unwrap();

        assert_eq!(&Buffer, Data);

        // - Send data from Client to Server
        let Data = b"fizzbuzz";

        Server_stream.write_all(Data).unwrap();

        let mut Buffer = [0; 8];
        let Size = Driver.Receive(Client, &mut Buffer).unwrap();

        assert_eq!(Size, Data.len());
        assert_eq!(&Buffer, Data);
    }

    #[test]
    fn test_TCP_send_receive_both_sides() {
        let Driver = Network_socket_driver_type::new();

        let Server_listener = New_socket_identifier(1.into());
        let Server_stream = New_socket_identifier(2.into());
        let Client = New_socket_identifier(3.into());

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
    fn test_UDP_send_to_receive_from_one_side() {
        let Driver = Network_socket_driver_type::new();

        let Socket_1 = New_socket_identifier(1.into());

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
        let Socket_1_address = Into_socketaddr(IP_1, Port_1);
        let Socket_2_address = Socket_2.local_addr().unwrap();
        let (IP_2, Port_2) = Into_IP_and_port(Socket_2_address);

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
    fn test_UDP_send_to_receive_from_both_sides() {
        let Driver = Network_socket_driver_type::new();

        let Socket_1_identifier = New_socket_identifier(1.into());
        let Socket_2_identifier = New_socket_identifier(2.into());

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
