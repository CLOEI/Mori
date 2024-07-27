use crate::types::{
    e_packet_type::EPacketType, e_tank_packet_type::ETankPacketType,
    tank_packet_type::TankPacketType,
};
use crate::utils::bytes;

use super::variant_handler;
use super::Bot;

use enet::{Packet, PacketMode, Peer};
use spdlog::info;

pub fn handle(bot: &mut Bot, peer: &mut Peer<()>, packet_type: EPacketType, data: &[u8]) {
    match packet_type {
        EPacketType::NetMessageServerHello => {
            info!("Received NetMessageServerHello");
            if bot.is_redirect {
                let message = format!(
                    "UUIDToken|{}\nprotocol|{}\nfhash|{}\nmac|{}\nrequestedName|{}\nhash2|{}\nfz|{}\nf|{}\nplayer_age|{}\ngame_version|{}\nlmode|{}\ncbits|{}\nrid|{}\nGDPR|{}\nhash|{}\ncategory|{}\ntoken|{}\ntotal_playtime|{}\ndoor_id|{}\nklv|{}\nmeta|{}\nplatformID|{}\ndeviceVersion|{}\nzf|{}\ncountry|{}\nuser|{}\nwk|{}\n",
                    bot.login_info.uuid, bot.login_info.protocol, bot.login_info.fhash, bot.login_info.mac, bot.login_info.requested_name, bot.login_info.hash2, bot.login_info.fz, bot.login_info.f, bot.login_info.player_age, bot.login_info.game_version, bot.login_info.lmode, bot.login_info.cbits, bot.login_info.rid, bot.login_info.gdpr, bot.login_info.hash, bot.login_info.category, bot.login_info.token, bot.login_info.total_playtime, bot.login_info.door_id, bot.login_info.klv, bot.login_info.meta, bot.login_info.platform_id, bot.login_info.device_version, bot.login_info.zf, bot.login_info.country, bot.login_info.user, bot.login_info.wk
                );
                bot.send_packet(peer, EPacketType::NetMessageGenericText, message);
            } else {
                let message = format!(
                    "protocol|{}\nltoken|{}\nplatformID|{}\n",
                    209, bot.token, "0,1,1"
                );
                bot.send_packet(peer, EPacketType::NetMessageGenericText, message);
            }
        }
        EPacketType::NetMessageGenericText => {
            info!("Received NetMessageGenericText");
        }
        EPacketType::NetMessageGameMessage => {
            let message = String::from_utf8_lossy(&data[4..]);
            info!("Received NetMessageGameMessage");
            info!("Message: {}", message);

            if message.contains("logon_fail") {
                bot.is_redirect = false;
                bot.disconnect(peer);
            }
            if message.contains("currently banned") {
                bot.is_banned = true;
                bot.is_running = false;
                bot.disconnect(peer);
            }
        }
        EPacketType::NetMessageGamePacket => {
            let tank_packet = map_slice_to_tank_packet_type(data);
            info!("Received Tank packet type: {:?}", tank_packet.packet_type);

            if tank_packet.packet_type == ETankPacketType::NetGamePacketCallFunction {
                variant_handler::handle(bot, peer, &tank_packet, &data[56..]);
            }
            if tank_packet.packet_type == ETankPacketType::NetGamePacketSendMapData {
                bot.world.parse(&data[56..]);
                bot.astar
                    .update(&bot.world.tiles, bot.world.width, bot.world.height);
            }
            if tank_packet.packet_type == ETankPacketType::NetGamePacketSendInventoryState {
                bot.inventory.parse(&data[56..]);
            }
            if tank_packet.packet_type == ETankPacketType::NetGamePacketPingRequest {
                let mut pkt = TankPacketType::new();
                pkt.packet_type = ETankPacketType::NetGamePacketPingReply;
                pkt.net_id = 0; // I'm not sure why it must be 0 instead of bot.net_id
                pkt.unk2 = 0;
                pkt.vector_x = 64.0;
                pkt.vector_y = 64.0;
                pkt.vector_x2 = 1000.0;
                pkt.vector_y2 = 250.0;

                let mut packet_data = Vec::new();
                packet_data
                    .extend_from_slice(&(EPacketType::NetMessageGamePacket as u32).to_le_bytes());
                packet_data.extend_from_slice(&(pkt.packet_type as u8).to_le_bytes());
                packet_data.extend_from_slice(&pkt.unk1.to_le_bytes());
                packet_data.extend_from_slice(&pkt.unk2.to_le_bytes());
                packet_data.extend_from_slice(&pkt.unk3.to_le_bytes());
                packet_data.extend_from_slice(&pkt.net_id.to_le_bytes());
                packet_data.extend_from_slice(&pkt.unk4.to_le_bytes());
                packet_data.extend_from_slice(&pkt.flags.to_le_bytes());
                packet_data.extend_from_slice(&pkt.unk6.to_le_bytes());
                packet_data.extend_from_slice(&pkt.value.to_le_bytes());
                packet_data.extend_from_slice(&pkt.vector_x.to_le_bytes());
                packet_data.extend_from_slice(&pkt.vector_y.to_le_bytes());
                packet_data.extend_from_slice(&pkt.vector_x2.to_le_bytes());
                packet_data.extend_from_slice(&pkt.vector_y2.to_le_bytes());
                packet_data.extend_from_slice(&pkt.unk12.to_le_bytes());
                packet_data.extend_from_slice(&pkt.int_x.to_le_bytes());
                packet_data.extend_from_slice(&pkt.int_y.to_le_bytes());
                packet_data.extend_from_slice(&pkt.extended_data_length.to_le_bytes());
                packet_data.extend_from_slice(&data[56..]);

                let pkt = Packet::new(&packet_data, PacketMode::ReliableSequenced).unwrap();
                peer.send_packet(pkt, 0).unwrap();
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
            let message = String::from_utf8_lossy(data);
            info!("Message: {}", message);
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
        flags: bytes::bytes_to_u32(&data[12..16]),
        unk6: bytes::bytes_to_u32(&data[16..20]),
        value: bytes::bytes_to_u32(&data[20..24]),
        vector_x: bytes::bytes_to_f32(&data[24..28]),
        vector_y: bytes::bytes_to_f32(&data[28..32]),
        vector_x2: bytes::bytes_to_f32(&data[32..36]),
        vector_y2: bytes::bytes_to_f32(&data[36..40]),
        unk12: bytes::bytes_to_f32(&data[40..44]),
        int_x: bytes::bytes_to_i32(&data[44..48]),
        int_y: bytes::bytes_to_i32(&data[48..52]),
        extended_data_length: bytes::bytes_to_u32(&data[52..56]),
    }
}
