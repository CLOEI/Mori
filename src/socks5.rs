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

        let mut relay_addr = Self::socks5_handshake(&mut control_stream, username, password)?;

        // If server returns 0.0.0.0, use the proxy's IP for the relay
        if relay_addr.ip().is_unspecified() {
            relay_addr.set_ip(proxy_addr.ip());
        }

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
            vec![0x05, 0x02, 0x00, 0x02]
        } else {
            vec![0x05, 0x01, 0x00]
        };

        stream.write_all(&auth_methods)?;

        let mut response = [0u8; 2];
        stream.read_exact(&mut response)?;

        if response[0] != 0x05 {
            return Err(Socks5Error::UnsupportedVersion);
        }

        match response[1] {
            0x00 => Ok(()),
            0x02 => Ok(()),
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
        auth_request.push(0x01);
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
        let request = [0x05, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        stream.write_all(&request)?;

        let mut response = [0u8; 10];
        stream.read_exact(&mut response[..4])?;

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
                stream.read_exact(&mut response[4..10])?;
                let ip = Ipv4Addr::new(response[4], response[5], response[6], response[7]);
                let port = u16::from_be_bytes([response[8], response[9]]);
                Ok(SocketAddr::from((ip, port)))
            }
            0x04 => {
                let mut ipv6_response = [0u8; 18];
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
        header.extend_from_slice(&[0x00, 0x00]);
        header.push(0x00);

        match target_addr {
            SocketAddr::V4(addr) => {
                header.push(0x01);
                header.extend_from_slice(&addr.ip().octets());
                header.extend_from_slice(&addr.port().to_be_bytes());
            }
            SocketAddr::V6(addr) => {
                header.push(0x04);
                header.extend_from_slice(&addr.ip().octets());
                header.extend_from_slice(&addr.port().to_be_bytes());
            }
        }

        header
    }

    fn parse_udp_header<'a>(&self, data: &'a [u8]) -> io::Result<(SocketAddr, &'a [u8])> {
        if data.len() < 10 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "UDP header too short",
            ));
        }

        if data[0] != 0x00 || data[1] != 0x00 {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid RSV field"));
        }

        if data[2] != 0x00 {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "Fragmentation not supported",
            ));
        }

        match data[3] {
            0x01 => {
                let ip = Ipv4Addr::new(data[4], data[5], data[6], data[7]);
                let port = u16::from_be_bytes([data[8], data[9]]);
                Ok((SocketAddr::from((ip, port)), &data[10..]))
            }
            0x04 => {
                if data.len() < 22 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "IPv6 header too short",
                    ));
                }
                let ip_bytes: [u8; 16] = data[4..20].try_into().unwrap();
                let ip = Ipv6Addr::from(ip_bytes);
                let port = u16::from_be_bytes([data[20], data[21]]);
                Ok((SocketAddr::from((ip, port)), &data[22..]))
            }
            0x03 => Err(io::Error::new(
                ErrorKind::Unsupported,
                "Domain name addresses not supported",
            )),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                "Unsupported address type",
            )),
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
        let mut packet = self.create_udp_header(address);
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
