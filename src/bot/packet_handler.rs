use crate::types::{
    e_packet_type::EPacketType, e_tank_packet_type::ETankPacketType,
    tank_packet_type::TankPacketType,
};
use crate::utils::bytes;

use super::variant_handler;
use super::Bot;

use enet::Peer;
use spdlog::info;

pub fn handle(bot: &Bot, peer: &mut Peer<()>, packet_type: EPacketType, data: &[u8]) {
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
            let tank_packet = map_slice_to_tank_packet_type(data);
            info!("Received Tank packet type: {:?}", tank_packet.packet_type);

            if tank_packet.packet_type == ETankPacketType::NetGamePacketCallFunction {
                variant_handler::handle(&bot, &peer, &tank_packet, &data[56..]);
            }
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
    TankPacketType {
        packet_type: ETankPacketType::from(data[0]),
        unk1: data[1],
        unk2: data[2],
        unk3: data[3],
        net_id: bytes::bytes_to_u32(&data[4..8]),
        unk4: bytes::bytes_to_u32(&data[8..12]),
        unk5: bytes::bytes_to_u32(&data[12..16]),
        unk6: bytes::bytes_to_u32(&data[16..20]),
        unk7: bytes::bytes_to_u32(&data[20..24]),
        unk8: bytes::bytes_to_f32(&data[24..28]),
        unk9: bytes::bytes_to_f32(&data[28..32]),
        unk10: bytes::bytes_to_f32(&data[32..36]),
        unk11: bytes::bytes_to_f32(&data[36..40]),
        unk12: bytes::bytes_to_f32(&data[40..44]),
        unk13: bytes::bytes_to_u32(&data[44..48]),
        unk14: bytes::bytes_to_u32(&data[48..52]),
        extended_data_length: bytes::bytes_to_u32(&data[52..56]),
    }
}
