use rusty_enet::{MTU_MAX, PacketReceived, SocketOptions};
use std::io::{self, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream, UdpSocket};
use std::time::Duration;

#[derive(Debug)]
pub enum Socks5Error {
    Io(io::Error),
    InvalidResponse,
    AuthenticationFailed,
    UnsupportedVersion,
    ConnectionRefused,
    NetworkUnreachable,
    HostUnreachable,
    ConnectionReset,
    CommandNotSupported,
    AddressTypeNotSupported,
    GeneralFailure,
}

impl From<io::Error> for Socks5Error {
    fn from(err: io::Error) -> Self {
        Socks5Error::Io(err)
    }
}

impl From<Socks5Error> for io::Error {
    fn from(err: Socks5Error) -> io::Error {
        match err {
            Socks5Error::Io(e) => e,
            Socks5Error::InvalidResponse => {
                io::Error::new(ErrorKind::InvalidData, "Invalid SOCKS5 response")
            }
            Socks5Error::AuthenticationFailed => {
                io::Error::new(ErrorKind::PermissionDenied, "SOCKS5 authentication failed")
            }
            Socks5Error::UnsupportedVersion => {
                io::Error::new(ErrorKind::Unsupported, "Unsupported SOCKS5 version")
            }
            Socks5Error::ConnectionRefused => {
                io::Error::new(ErrorKind::ConnectionRefused, "SOCKS5 connection refused")
            }
            Socks5Error::NetworkUnreachable => {
                io::Error::new(ErrorKind::NetworkUnreachable, "Network unreachable")
            }
            Socks5Error::HostUnreachable => io::Error::new(ErrorKind::Other, "Host unreachable"),
            Socks5Error::ConnectionReset => {
                io::Error::new(ErrorKind::ConnectionReset, "Connection reset")
            }
            Socks5Error::CommandNotSupported => {
                io::Error::new(ErrorKind::Unsupported, "Command not supported")
            }
            Socks5Error::AddressTypeNotSupported => {
                io::Error::new(ErrorKind::Unsupported, "Address type not supported")
            }
            Socks5Error::GeneralFailure => {
                io::Error::new(ErrorKind::Other, "General SOCKS5 failure")
            }
        }
    }
}

pub struct Socks5UdpSocket {
    udp_socket: UdpSocket,
    _control_stream: TcpStream,
    relay_addr: SocketAddr,
}

impl Socks5UdpSocket {
    pub fn bind_through_proxy(
        local_addr: SocketAddr,
        proxy_addr: SocketAddr,
        username: Option<&str>,
        password: Option<&str>,
    ) -> io::Result<Self> {
        let mut control_stream = TcpStream::connect_timeout(&proxy_addr, Duration::from_secs(10))?;

        let relay_addr = Self::socks5_handshake(&mut control_stream, username, password)?;

        let udp_socket = UdpSocket::bind(local_addr)?;
        udp_socket.set_nonblocking(true)?;

        Ok(Self {
            udp_socket,
            _control_stream: control_stream,
            relay_addr,
        })
    }

    fn socks5_handshake(
        stream: &mut TcpStream,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<SocketAddr, Socks5Error> {
        Self::negotiate_auth_method(stream, username.is_some() && password.is_some())?;

        if username.is_some() && password.is_some() {
            Self::authenticate(stream, username.unwrap(), password.unwrap())?;
        }

        Self::udp_associate(stream)
    }

    fn negotiate_auth_method(stream: &mut TcpStream, use_auth: bool) -> Result<(), Socks5Error> {
        let auth_methods = if use_auth {
            vec![0x05, 0x02, 0x00, 0x02] // Version 5, 2 methods: No auth (0x00), Username/Password (0x02)
        } else {
            vec![0x05, 0x01, 0x00] // Version 5, 1 method: No auth (0x00)
        };

        stream.write_all(&auth_methods)?;

        let mut response = [0u8; 2];
        stream.read_exact(&mut response)?;

        if response[0] != 0x05 {
            return Err(Socks5Error::UnsupportedVersion);
        }

        match response[1] {
            0x00 => Ok(()), // No authentication required
            0x02 => Ok(()), // Username/password authentication required
            0xFF => Err(Socks5Error::AuthenticationFailed),
            _ => Err(Socks5Error::InvalidResponse),
        }
    }

    fn authenticate(
        stream: &mut TcpStream,
        username: &str,
        password: &str,
    ) -> Result<(), Socks5Error> {
        let mut auth_request = Vec::new();
        auth_request.push(0x01); // Version 1 for username/password auth
        auth_request.push(username.len() as u8);
        auth_request.extend_from_slice(username.as_bytes());
        auth_request.push(password.len() as u8);
        auth_request.extend_from_slice(password.as_bytes());

        stream.write_all(&auth_request)?;

        let mut response = [0u8; 2];
        stream.read_exact(&mut response)?;

        if response[0] != 0x01 || response[1] != 0x00 {
            return Err(Socks5Error::AuthenticationFailed);
        }

        Ok(())
    }

    fn udp_associate(stream: &mut TcpStream) -> Result<SocketAddr, Socks5Error> {
        let request = [
            0x05, // SOCKS version 5
            0x03, // UDP ASSOCIATE command
            0x00, // Reserved
            0x01, // IPv4 address type
            0x00, 0x00, 0x00, 0x00, // Address (0.0.0.0)
            0x00, 0x00, // Port
        ];

        stream.write_all(&request)?;

        let mut response = [0u8; 10]; // Minimum response size for IPv4
        stream.read_exact(&mut response[..4])?; // Read first 4 bytes

        if response[0] != 0x05 {
            return Err(Socks5Error::UnsupportedVersion);
        }

        match response[1] {
            0x00 => {}
            0x01 => return Err(Socks5Error::GeneralFailure),
            0x02 => return Err(Socks5Error::ConnectionRefused),
            0x03 => return Err(Socks5Error::NetworkUnreachable),
            0x04 => return Err(Socks5Error::HostUnreachable),
            0x05 => return Err(Socks5Error::ConnectionRefused),
            0x06 => return Err(Socks5Error::ConnectionReset),
            0x07 => return Err(Socks5Error::CommandNotSupported),
            0x08 => return Err(Socks5Error::AddressTypeNotSupported),
            _ => return Err(Socks5Error::InvalidResponse),
        }

        match response[3] {
            0x01 => {
                // IPv4
                stream.read_exact(&mut response[4..10])?;
                let ip = Ipv4Addr::new(response[4], response[5], response[6], response[7]);
                let port = u16::from_be_bytes([response[8], response[9]]);
                Ok(SocketAddr::from((ip, port)))
            }
            0x04 => {
                // IPv6
                let mut ipv6_response = [0u8; 18]; // 16 bytes for IPv6 + 2 for port
                stream.read_exact(&mut ipv6_response)?;
                let ip_bytes: [u8; 16] = ipv6_response[..16].try_into().unwrap();
                let ip = Ipv6Addr::from(ip_bytes);
                let port = u16::from_be_bytes([ipv6_response[16], ipv6_response[17]]);
                Ok(SocketAddr::from((ip, port)))
            }
            0x03 => Err(Socks5Error::InvalidResponse),
            _ => Err(Socks5Error::AddressTypeNotSupported),
        }
    }

    fn create_udp_header(&self, target_addr: SocketAddr) -> Vec<u8> {
        let mut header = Vec::new();

        // RSV (2 bytes) - Reserved, must be 0x0000
        header.extend_from_slice(&[0x00, 0x00]);

        // FRAG (1 byte) - Fragment number, 0x00 for standalone datagram
        header.push(0x00);

        match target_addr {
            SocketAddr::V4(addr) => {
                // ATYP (1 byte) - IPv4 address type
                header.push(0x01);

                // DST.ADDR (4 bytes) - IPv4 address
                header.extend_from_slice(&addr.ip().octets());

                // DST.PORT (2 bytes) - Port in network byte order
                header.extend_from_slice(&addr.port().to_be_bytes());
            }
            SocketAddr::V6(addr) => {
                // ATYP (1 byte) - IPv6 address type
                header.push(0x04);

                // DST.ADDR (16 bytes) - IPv6 address
                header.extend_from_slice(&addr.ip().octets());

                // DST.PORT (2 bytes) - Port in network byte order
                header.extend_from_slice(&addr.port().to_be_bytes());
            }
        }

        header
    }

    fn parse_udp_header<'a>(&self, data: &'a [u8]) -> io::Result<(SocketAddr, &'a [u8])> {
        if data.len() < 10 {
            // Minimum header size for IPv4
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "UDP header too short",
            ));
        }

        // Check RSV field (should be 0x0000)
        if data[0] != 0x00 || data[1] != 0x00 {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid RSV field"));
        }

        // Check FRAG field
        if data[2] != 0x00 {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "Fragmentation not supported",
            ));
        }

        let atyp = data[3];
        match atyp {
            0x01 => {
                // IPv4
                if data.len() < 10 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "IPv4 header too short",
                    ));
                }

                let ip = Ipv4Addr::new(data[4], data[5], data[6], data[7]);
                let port = u16::from_be_bytes([data[8], data[9]]);
                let addr = SocketAddr::from((ip, port));
                let payload = &data[10..];

                Ok((addr, payload))
            }
            0x04 => {
                // IPv6
                if data.len() < 22 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "IPv6 header too short",
                    ));
                }

                let ip_bytes: [u8; 16] = data[4..20].try_into().unwrap();
                let ip = Ipv6Addr::from(ip_bytes);
                let port = u16::from_be_bytes([data[20], data[21]]);
                let addr = SocketAddr::from((ip, port));
                let payload = &data[22..];

                Ok((addr, payload))
            }
            0x03 => {
                // Domain name
                if data.len() < 5 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "Domain name header too short",
                    ));
                }

                let domain_len = data[4] as usize;
                let total_header_len = 4 + 1 + domain_len + 2; // RSV + FRAG + ATYP + len + domain + port

                if data.len() < total_header_len {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "Domain name header incomplete",
                    ));
                }

                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "Domain name addresses not supported",
                ));
            }
            _ => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "Unsupported address type",
                ));
            }
        }
    }
}

impl rusty_enet::Socket for Socks5UdpSocket {
    type Address = SocketAddr;
    type Error = io::Error;

    fn init(&mut self, _socket_options: SocketOptions) -> Result<(), Self::Error> {
        Ok(())
    }

    fn send(&mut self, address: Self::Address, buffer: &[u8]) -> Result<usize, Self::Error> {
        let header = self.create_udp_header(address);

        let mut packet = header;
        packet.extend_from_slice(buffer);

        match self.udp_socket.send_to(&packet, self.relay_addr) {
            Ok(sent) => {
                if sent >= packet.len() {
                    Ok(buffer.len())
                } else {
                    Ok(0)
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(e),
        }
    }

    fn receive(
        &mut self,
        buffer: &mut [u8; MTU_MAX],
    ) -> Result<Option<(Self::Address, PacketReceived)>, Self::Error> {
        match self.udp_socket.recv_from(buffer) {
            Ok((size, _source)) => {
                let received_data = &buffer[..size];
                match self.parse_udp_header(received_data) {
                    Ok((real_addr, payload)) => {
                        let payload_len = payload.len();
                        if payload_len <= MTU_MAX {
                            let payload_offset =
                                payload.as_ptr() as usize - buffer.as_ptr() as usize;
                            if payload_offset > 0 {
                                buffer.copy_within(payload_offset..payload_offset + payload_len, 0);
                            }
                            Ok(Some((real_addr, PacketReceived::Complete(payload_len))))
                        } else {
                            Ok(None)
                        }
                    }
                    Err(_) => Ok(None),
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn address(&self) -> Self::Address {
        self.udp_socket
            .local_addr()
            .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap())
    }
}
