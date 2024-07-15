mod packet_handler;
mod variant_handler;

use crate::types::e_packet_type::EPacketType;
use crate::types::login_info::LoginInfo;

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
    pub parsed_server_data: HashMap<String, String>,
    pub token: String,
    pub is_redirect: bool,
    pub is_running: bool,
    pub server: Server,
    pub login_info: LoginInfo,
}

impl Bot {
    pub fn new() -> Bot {
        Bot {
            username: String::new(),
            parsed_server_data: HashMap::new(),
            token: String::new(),
            is_redirect: false,
            is_running: false,
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
        let mut packet_data = Vec::new();
        packet_data.extend_from_slice(&(packet_type as u32).to_le_bytes());
        packet_data.extend_from_slice(&message.as_bytes());
        let pkt = Packet::new(&packet_data, PacketMode::ReliableSequenced).unwrap();
        peer.send_packet(pkt, 0).unwrap();
    }

    pub fn disconnect(&self, peer: &mut Peer<()>) {
        peer.disconnect(0);
    }
}
