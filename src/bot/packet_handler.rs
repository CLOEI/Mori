use crate::types::e_packet_type::EPacketType;

use super::Bot;

use enet::{Packet, PacketMode, Peer};
use spdlog::info;

pub fn handler(bot: &Bot, peer: &mut Peer<()>, packet_type: EPacketType, data: &[u8]) {
    match packet_type {
        EPacketType::NetMessageServerHello => {
            info!("Received NetMessageServerHello");
            let message = format!(
                "protocol|{}\nltoken|{}\nplatformID|{}\n",
                208, bot.token, "0,1,1"
            );
            bot.send_packet(peer, EPacketType::NetMessageGenericText, message);
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
