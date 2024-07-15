mod packet_handler;
mod variant_handler;

use crate::types::login_info::LoginInfo;
use crate::utils::proton::hash_string;
use crate::utils::random::random_hex;
use crate::{types::e_packet_type::EPacketType, utils::proton::generate_klv};

use std::collections::HashMap;

use byteorder::{ByteOrder, LittleEndian};
use enet::*;
use spdlog::info;

pub struct Server {
    ip: String,
    port: String,
}

pub struct Bot {
    pub username: String,
    pub current_world: String,
    pub net_id: u32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub gems: i32,
    pub parsed_server_data: HashMap<String, String>,
    pub token: String,
    pub is_redirect: bool,
    pub is_running: bool,
    pub is_banned: bool,
    pub server: Server,
    pub login_info: LoginInfo,
}

impl Bot {
    pub fn new() -> Bot {
        Bot {
            username: String::new(),
            current_world: String::new(),
            net_id: 0,
            pos_x: 0.0,
            pos_y: 0.0,
            gems: 0,
            parsed_server_data: HashMap::new(),
            token: String::new(),
            is_redirect: false,
            is_running: false,
            is_banned: false,
            server: Server {
                ip: String::new(),
                port: String::new(),
            },
            login_info: LoginInfo::new(),
        }
    }
}

impl Bot {
    pub fn login(&mut self) {
        self.is_running = true;
        self.login_info.meta = self.parsed_server_data["meta"].clone();

        self.login_info.klv = generate_klv(
            &self.login_info.protocol,
            &self.login_info.game_version,
            &self.login_info.rid,
        );
        self.login_info.hash =
            hash_string(format!("{}RT", self.login_info.mac).as_str()).to_string();
        self.login_info.hash2 =
            hash_string(format!("{}RT", random_hex(16, true)).as_str()).to_string();

        let enet = Enet::new().expect("Failed to initialize ENet");

        while self.is_running {
            let mut enet_host = Enet::create_host::<()>(
                &enet,
                None,
                1,
                ChannelLimit::Limited(1),
                BandwidthLimit::Unlimited,
                BandwidthLimit::Unlimited,
                true,
            )
            .expect("Failed to create ENet host");
            if self.is_redirect {
                info!("Redirecting to {}:{}...", self.server.ip, self.server.port);
                enet_host
                    .connect(
                        &Address::new(
                            self.server.ip.parse().unwrap(),
                            self.server.port.parse().unwrap(),
                        ),
                        2,
                        0,
                    )
                    .expect("Failed to connect to the server");
            } else {
                info!(
                    "Connecting to {}:{}",
                    self.parsed_server_data["server"], self.parsed_server_data["port"]
                );
                enet_host
                    .connect(
                        &Address::new(
                            self.parsed_server_data["server"].parse().unwrap(),
                            self.parsed_server_data["port"].parse().unwrap(),
                        ),
                        2,
                        0,
                    )
                    .expect("Failed to connect to the server");
            }

            loop {
                match enet_host.service(1000).expect("Service failed") {
                    Some(Event::Connect(ref peer)) => {
                        info!("Connected to the server");
                    }
                    Some(Event::Disconnect(..)) => {
                        info!("Disconnected from the server");
                        break;
                    }
                    Some(Event::Receive {
                        ref packet,
                        ref mut sender,
                        ..
                    }) => {
                        let data = packet.data();
                        if data.len() < 4 {
                            continue;
                        }
                        let packet_id = LittleEndian::read_u32(&data[0..4]);
                        let packet_type = EPacketType::from(packet_id);
                        packet_handler::handle(self, sender, packet_type, &data[4..]);
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn send_packet(&self, peer: &mut Peer<()>, packet_type: EPacketType, message: String) {
        if packet_type == EPacketType::NetMessageGamePacket {
            // TODO: Implement this
        } else {
            let mut packet_data = Vec::new();
            packet_data.extend_from_slice(&(packet_type as u32).to_le_bytes());
            packet_data.extend_from_slice(&message.as_bytes());
            let pkt = Packet::new(&packet_data, PacketMode::ReliableSequenced).unwrap();
            peer.send_packet(pkt, 0).unwrap();
        }
    }

    pub fn disconnect(&self, peer: &mut Peer<()>) {
        peer.disconnect(0);
    }
}
