use crate::Socks5Config;
use crate::socks5_udp::Socks5UdpSocket;
use rusty_enet::{EventNoRef, HostSettings, Packet, PeerID};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Mutex;

enum SessionHost {
    Direct(rusty_enet::Host<UdpSocket>),
    Socks5(rusty_enet::Host<Socks5UdpSocket>),
}

pub struct NetworkSession {
    host: Mutex<SessionHost>,
    peer_id: Mutex<Option<PeerID>>,
}

impl NetworkSession {
    pub fn new(local_addr: SocketAddr, socks5_config: Option<Socks5Config>) -> Self {
        let host = match socks5_config {
            Some(cfg) => {
                let socks5_socket = Socks5UdpSocket::bind_through_proxy(
                    local_addr,
                    cfg.proxy_addr,
                    cfg.username.as_deref(),
                    cfg.password.as_deref(),
                )
                .expect("Failed to create SOCKS5 socket");

                let host = rusty_enet::Host::<Socks5UdpSocket>::new(
                    socks5_socket,
                    HostSettings {
                        peer_limit: 1,
                        channel_limit: 2,
                        compressor: Some(Box::new(rusty_enet::RangeCoder::new())),
                        checksum: Some(Box::new(rusty_enet::crc32)),
                        using_new_packet: true,
                        ..Default::default()
                    },
                )
                .expect("Failed to create SOCKS5 host");

                SessionHost::Socks5(host)
            }
            None => {
                let socket = UdpSocket::bind(local_addr).expect("Failed to bind UDP socket");

                let host = rusty_enet::Host::<UdpSocket>::new(
                    socket,
                    HostSettings {
                        peer_limit: 1,
                        channel_limit: 2,
                        compressor: Some(Box::new(rusty_enet::RangeCoder::new())),
                        checksum: Some(Box::new(rusty_enet::crc32)),
                        using_new_packet: true,
                        ..Default::default()
                    },
                )
                .expect("Failed to create direct host");

                SessionHost::Direct(host)
            }
        };

        Self {
            host: Mutex::new(host),
            peer_id: Mutex::new(None),
        }
    }

    pub fn connect(&self, address: SocketAddr) {
        let mut host = self.host.lock().unwrap();
        match &mut *host {
            SessionHost::Direct(host) => {
                if let Err(err) = host.connect(address, 2, 0) {
                    panic!("Failed to connect to server: {}", err);
                }
            }
            SessionHost::Socks5(host) => {
                if let Err(err) = host.connect(address, 2, 0) {
                    panic!("Failed to connect to server: {}", err);
                }
            }
        }
    }

    pub fn disconnect(&self) {
        let peer_id = self.peer_id.lock().unwrap().take();
        if let Some(peer_id) = peer_id {
            let mut host = self.host.lock().unwrap();
            match &mut *host {
                SessionHost::Direct(host) => {
                    let _ = host.peer_mut(peer_id).disconnect(0);
                }
                SessionHost::Socks5(host) => {
                    let _ = host.peer_mut(peer_id).disconnect(0);
                }
            }
        }
    }

    pub fn send(&self, packet: Packet) -> bool {
        let peer_id = {
            let guard = self.peer_id.lock().unwrap();
            match *guard {
                Some(peer_id) => peer_id,
                None => return false,
            }
        };

        let mut host = self.host.lock().unwrap();
        match &mut *host {
            SessionHost::Direct(host) => {
                let peer = host.peer_mut(peer_id);
                if let Err(err) = peer.send(0, &packet) {
                    println!("Failed to send packet: {}", err);
                    return false;
                }
                true
            }
            SessionHost::Socks5(host) => {
                let peer = host.peer_mut(peer_id);
                if let Err(err) = peer.send(0, &packet) {
                    println!("Failed to send packet: {}", err);
                    return false;
                }
                true
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.peer_id.lock().unwrap().is_some()
    }

    pub fn update_ping(&self) -> Option<u32> {
        let peer_id = {
            let guard = self.peer_id.lock().unwrap();
            guard.clone()?
        };

        let mut host = self.host.lock().unwrap();
        let ping = match &mut *host {
            SessionHost::Direct(host) => host.peer_mut(peer_id).round_trip_time().as_millis(),
            SessionHost::Socks5(host) => host.peer_mut(peer_id).round_trip_time().as_millis(),
        };

        Some(ping as u32)
    }

    pub fn service(&self) -> Option<EventNoRef> {
        let mut host = self.host.lock().unwrap();
        match &mut *host {
            SessionHost::Direct(host) => host.service().ok().flatten().map(|e| e.no_ref()),
            SessionHost::Socks5(host) => host.service().ok().flatten().map(|e| e.no_ref()),
        }
    }

    pub fn set_peer_id(&self, peer: Option<PeerID>) {
        let mut guard = self.peer_id.lock().unwrap();
        *guard = peer;
    }

    pub fn peer_id(&self) -> Option<PeerID> {
        self.peer_id.lock().unwrap().clone()
    }
}
