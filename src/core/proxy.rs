use rusty_enet::{PacketReceived, Socket, SocketOptions, MTU_MAX};
use socks::{Socks5Datagram, TargetAddr};
use std::io;
use std::net::{SocketAddr, UdpSocket};

pub struct Socks5UdpSocket {
    pub inner: Socks5Datagram,
}

impl Socks5UdpSocket {
    pub fn new(inner: Socks5Datagram) -> Self {
        Socks5UdpSocket { inner }
    }
}

impl Socket for Socks5UdpSocket {
    type Address = SocketAddr;
    type Error = io::Error;

    fn init(&mut self, _socket_options: SocketOptions) -> Result<(), io::Error> {
        self.inner.get_mut().set_nonblocking(true)?;
        self.inner.get_mut().set_broadcast(true)?;
        Ok(())
    }

    fn send(&mut self, address: Self::Address, buffer: &[u8]) -> Result<usize, Self::Error> {
        self.inner.send_to(buffer, address).map_err(|e| e.into())
    }

    fn receive(
        &mut self,
        buffer: &mut [u8; MTU_MAX],
    ) -> Result<Option<(Self::Address, PacketReceived)>, Self::Error> {
        match self.inner.recv_from(buffer) {
            Ok((size, addr)) => {
                let socket_addr = match addr {
                    TargetAddr::Ip(socket_addr) => socket_addr,
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid address type",
                        ))
                    }
                };
                let packet = PacketReceived::Complete(size);
                Ok(Some((socket_addr, packet)))
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }
}

pub enum SocketType {
    Socks5(Socks5UdpSocket),
    Udp(UdpSocket),
}

impl Socket for SocketType {
    type Address = SocketAddr;
    type Error = io::Error;

    fn init(&mut self, socket_options: SocketOptions) -> Result<(), Self::Error> {
        match self {
            SocketType::Socks5(s) => s.init(socket_options),
            SocketType::Udp(u) => u.init(socket_options),
        }
    }

    fn send(&mut self, address: Self::Address, buffer: &[u8]) -> Result<usize, Self::Error> {
        match self {
            SocketType::Socks5(s) => s.send(address, buffer),
            SocketType::Udp(u) => u.send(address, buffer),
        }
    }

    fn receive(
        &mut self,
        buffer: &mut [u8; MTU_MAX],
    ) -> Result<Option<(Self::Address, PacketReceived)>, Self::Error> {
        match self {
            SocketType::Socks5(s) => s.receive(buffer),
            SocketType::Udp(u) => u.receive(buffer),
        }
    }
}
