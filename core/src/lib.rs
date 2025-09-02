use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::sync::{Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use rusty_enet::Packet;
use crate::types::bot::{Info, State};
use crate::types::login_info::LoginInfo;
use crate::types::net_message::NetMessage;

mod server;
mod types;
mod utils;
mod login;
mod packet_handler;
mod variant_handler;

pub struct Bot {
    pub host: Mutex<rusty_enet::Host<UdpSocket>>,
    pub peer_id: Mutex<Option<rusty_enet::PeerID>>,
    pub info: Info,
    pub state: State,
    pub position: RwLock<(f32, f32)>,
    pub logs: RwLock<Vec<String>>,
    pub duration: Mutex<Instant>,
}

impl Bot {
    pub fn new(payload: Vec<String>) -> Self {
        let socket = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))).unwrap();
        let host = rusty_enet::Host::<UdpSocket>::new(
            socket,
            rusty_enet::HostSettings {
                peer_limit: 1,
                channel_limit: 2,
                compressor: Some(Box::new(rusty_enet::RangeCoder::new())),
                checksum: Some(Box::new(rusty_enet::crc32)),
                using_new_packet: true,
                ..Default::default()
            },
        ).expect("Failed to create host");

        Self {
            host: Mutex::new(host),
            peer_id: Mutex::new(None),
            info: Info {
                payload,
                login_method: types::bot::ELoginMethod::default(),
                login_info: Mutex::new(None),
                server_data: Mutex::new(None),
                dashboard_links: Mutex::new(None),
            },
            position: RwLock::new((0.0, 0.0)),
            state: State {
                is_running: Mutex::new(true),
                is_redirecting: Mutex::new(false),
                hack_type: Mutex::new(0),
            },
            logs: RwLock::new(Vec::new()),
            duration: Mutex::new(Instant::now()),
        }
    }

    pub fn logon(&self, data: Option<&str>) {
        if data.is_some() {
            todo!("Implement logon with pre-existing credentials");
        } else {
            let mut info = self.info.login_info.lock().unwrap();
            *info = Some(LoginInfo::new());
        }
        self.process_event();
    }

    pub fn connect_to_server(&self) {
        if !*self.state.is_redirecting.lock().unwrap() {
            {
                let mut login_info = self.info.login_info.lock().unwrap();
                let info_data = login_info.as_mut().expect("Login info not set");
                let server_data = server::get_server_data(false, info_data);
                match server_data {
                    Ok(data) => {
                        info_data.meta = data.meta.clone();
                        let mut server = self.info.server_data.lock().unwrap();
                        *server = Some(data.clone());
                        let dashboard_data = server::get_dashboard(&data.loginurl, info_data).expect("Failed to get dashboard data");
                        let mut dashboard = self.info.dashboard_links.lock().unwrap();
                        *dashboard = Some(dashboard_data);
                    }
                    Err(e) => {
                        todo!("Handle error: {}", e);
                    }
                }
            }
            self.get_token();
        }

        let server_data = self.info.server_data.lock().unwrap();
        let server = server_data.as_ref().expect("Server data not set");

        let socket_address = SocketAddr::from_str(&format!("{}:{}", server.server, server.port)).unwrap();

        let mut host = self.host.lock().unwrap();
        match host.connect(socket_address, 2, 0) {
            Err(err) => {
                panic!("Failed to connect to server: {}", err);
            }
            _ => {}
        }
    }

    pub fn get_token(&self) {
        let dashboard_links = self.info.dashboard_links.lock().unwrap();
        let urls = dashboard_links.as_ref();
        let token = match self.info.login_method {
            types::bot::ELoginMethod::LEGACY => {
                let payload = self.info.payload.clone();
                let username = payload.get(0).expect("Username not found");
                let password = payload.get(1).expect("Password not found");
                let growtopia_url = urls.as_ref().and_then(|links| links.growtopia.clone());
                login::get_legacy_token(&growtopia_url.unwrap(), username, password).expect("Failed to get legacy token")
            },
            _ => {
                todo!("Implement token retrieval for different login methods");
            }
        };

        let mut login_info_lock = self.info.login_info.lock().unwrap();
        let login_info = login_info_lock.as_mut().expect("Login info not set");
        login_info.ltoken = token.clone();
    }

    pub fn send_packet(
        &self,
        packet_type: NetMessage,
        packet_data: &[u8],
        extended_data: Option<&[u8]>,
        reliable: bool,
    ) {
        const MAX_PACKET_SIZE: usize = 1_000_000;

        if packet_data.len() > MAX_PACKET_SIZE {
            println!("Error: Attempted to send huge packet of size {}", packet_data.len());
            return;
        }

        let mut final_payload = Vec::new();
        let mut is_special_case = false;

        if let NetMessage::GamePacket = packet_type {
            if packet_data.len() >= 16 {
                let flags_bytes: [u8; 4] = packet_data[12..16].try_into().expect("Slice with incorrect length");
                let flags = u32::from_le_bytes(flags_bytes);

                if (flags & 8) != 0 {
                    is_special_case = true;
                }
            }
        }

        final_payload.extend_from_slice(&(packet_type as u32).to_le_bytes());
        final_payload.extend_from_slice(packet_data);

        if is_special_case {
            if let Some(ext_data) = extended_data {
                if packet_data.len() >= 56 {
                    let len_bytes: [u8; 4] = packet_data[52..56].try_into().expect("Slice with incorrect length");
                    let extended_len = u32::from_le_bytes(len_bytes) as usize;

                    final_payload.extend_from_slice(&ext_data[..extended_len]);
                }
            }
        }

        let enet_packet = if reliable {
            Packet::reliable(final_payload)
        } else {
            Packet::unreliable(final_payload)
        };

        let peer_id_guard = self.peer_id.lock().expect("Failed to lock peer_id");
        let Some(peer_id) = *peer_id_guard else {
            println!("Cannot send packet: No active peer connection.");
            return;
        };

        let mut host_guard = self.host.lock().expect("Failed to lock host");
        let peer = host_guard.peer_mut(peer_id);

        if let Err(err) = peer.send(0, &enet_packet) {
            println!("Failed to send packet: {}", err);
        }
    }
    
    fn disconnect(&self) {
        let peer_id = self.peer_id.lock().unwrap().clone();
        if let Some(peer_id) = peer_id {
            if let Ok(mut host) = self.host.lock() {
                let peer = host.peer_mut(peer_id);
                peer.disconnect(0);
            }
        }
    }

    fn process_event(&self) {
        let is_running = {
            let running = self.state.is_running.lock().unwrap();
            *running
        };

        while is_running {
            self.connect_to_server();

            loop {
                let event = {
                    let mut host = self.host.lock().unwrap();
                    let event = host.service();
                    event.ok().flatten().map(|e| e.no_ref())
                };

                if let Some(event) = event {
                    match event {
                        rusty_enet::EventNoRef::Connect { peer, .. } => {
                            println!("Connected to server");
                            let mut peer_id_lock = self.peer_id.lock().unwrap();
                            *peer_id_lock = Some(peer);
                        }
                        rusty_enet::EventNoRef::Receive { peer, channel_id, packet } => {
                            let data = packet.data();
                            if data.len() < 4 {
                                continue;
                            }
                            packet_handler::handle(self, data);
                        }
                        rusty_enet::EventNoRef::Disconnect { peer, data } => {
                            println!("Disconnected from server");
                            break;
                        }
                    }
                }
            }
        }
    }
}