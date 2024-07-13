use crate::types::e_packet_type::EPacketType;

use std::{collections::HashMap, sync::Arc};

use byteorder::{ByteOrder, LittleEndian};
use enet::*;
use spdlog::info;

pub struct Bot {
    pub parsed_server_data: HashMap<String, String>,
    pub token: String,
}

impl Bot {
    pub fn new() -> Bot {
        Bot {
            parsed_server_data: HashMap::new(),
            token: String::new(),
        }
    }
}

impl Bot {
    pub fn login(&self) {
        let enet = Enet::new().expect("Failed to initialize ENet");

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

        info!(
            "Connecting to {}:{}",
            self.parsed_server_data["server"], self.parsed_server_data["port"]
        );
        let server = self.parsed_server_data["server"]
            .parse()
            .expect("Failed to parse server address");
        let port = self.parsed_server_data["port"]
            .parse()
            .expect("Failed to parse server port");
        enet_host
            .connect(&Address::new(server, port), 2, 0)
            .expect("Failed to connect to the server");

        loop {
            match enet_host.service(1000).expect("Service failed") {
                Some(Event::Connect(ref peer)) => {
                    info!("Connected to the server");
                }
                Some(Event::Disconnect(..)) => info!("Disconnected from the server"),
                Some(Event::Receive {
                    ref packet,
                    ref mut sender,
                    channel_id,
                }) => {
                    let data = packet.data();
                    let packet_id = LittleEndian::read_u32(&data[0..4]);
                    let packet_type = EPacketType::from(packet_id);

                    match packet_type {
                        EPacketType::NetMessageServerHello => {
                            info!("Received NetMessageServerHello");
                            let mut packet_data = Vec::new();
                            packet_data.extend_from_slice(
                                &(EPacketType::NetMessageGenericText as u32).to_le_bytes(),
                            );
                            let message = format!(
                                "protocol|{}\nltoken|{}\nplatformID|{}\n",
                                208, self.token, "0,1,1"
                            );
                            packet_data.extend_from_slice(&message.as_bytes());
                            let pkt =
                                Packet::new(&packet_data, PacketMode::ReliableSequenced).unwrap();
                            sender.send_packet(pkt, channel_id).unwrap();
                        }
                        EPacketType::NetMessageGenericText => {
                            info!("Received NetMessageGenericText");
                        }
                        EPacketType::NetMessageGameMessage => {
                            info!("Received NetMessageGameMessage");
                        }
                        EPacketType::NetMessageGamePacket => {
                            info!("Received NetMessageGamePacket");
                        }
                        EPacketType::NetMessageError => {
                            info!("Received NetMessageError");
                        }
                        EPacketType::NetMessageTrack => {
                            info!("Received NetMessageTrack");
                        }
                        EPacketType::NetMessageClientLogRequest => {
                            info!("Received NetMessageClientLogRequest");
                        }
                        EPacketType::NetMessageClientLogResponse => {
                            info!("Received NetMessageClientLogResponse");
                        }
                        EPacketType::NetMessageMax => {
                            info!("Received NetMessageMax");
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }
    }
}
