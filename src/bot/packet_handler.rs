use crate::types::{
    e_packet_type::EPacketType, e_tank_packet_type::ETankPacketType,
    tank_packet_type::TankPacketType,
};

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

fn map_slice_to_tank_packet_type(data: &[u8]) -> TankPacketType {
    let packet_type = TankPacketType {
        packet_type: ETankPacketType::from(data[0]),
        unk1: data[1],
        unk2: data[2],
        unk3: data[3],
        net_id: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        unk4: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        unk5: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
        unk6: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
        unk7: u32::from_le_bytes([data[20], data[21], data[22], data[23]]),
        unk8: f32::from_le_bytes([data[24], data[25], data[26], data[27]]),
        unk9: f32::from_le_bytes([data[28], data[29], data[30], data[31]]),
        unk10: f32::from_le_bytes([data[32], data[33], data[34], data[35]]),
        unk11: f32::from_le_bytes([data[36], data[37], data[38], data[39]]),
        unk12: f32::from_le_bytes([data[40], data[41], data[42], data[43]]),
        unk13: u32::from_le_bytes([data[44], data[45], data[46], data[47]]),
        unk14: u32::from_le_bytes([data[48], data[49], data[50], data[51]]),
        extended_data_length: u32::from_le_bytes([data[52], data[53], data[54], data[55]]),
    };
    packet_type
}
