use crate::types::{
    e_packet_type::EPacketType, e_tank_packet_type::ETankPacketType,
    tank_packet_type::TankPacketType,
};
use crate::utils::proton::{generate_klv, hash_string};
use crate::utils::random::random_hex;
use crate::utils::{bytes, proton};

use super::variant_handler;
use super::Bot;

use enet::Peer;
use spdlog::info;

pub fn handle(bot: &mut Bot, peer: &mut Peer<()>, packet_type: EPacketType, data: &[u8]) {
    match packet_type {
        EPacketType::NetMessageServerHello => {
            info!("Received NetMessageServerHello");
            if bot.is_redirect {
                println!("hello");
                bot.login_info.klv = generate_klv(
                    &bot.login_info.protocol,
                    &bot.login_info.game_version,
                    &bot.login_info.rid,
                );
                bot.login_info.hash =
                    hash_string(format!("{}RT", bot.login_info.mac).as_str()).to_string();
                bot.login_info.hash2 =
                    hash_string(format!("{}RT", random_hex(16, true)).as_str()).to_string();
                let message = format!(
                    "UUIDToken|{}\nprotocol|{}\nfhash|{}\nmac|{}\nrequestedName|{}\nhash2|{}\nfz|{}\nf|{}\nplayer_age|{}\ngame_version|{}\nlmode|{}\ncbits|{}\nrid|{}\nGDPR|{}\nhash|{}\ncategory|{}\ntoken|{}\ntotal_playtime|{}\ndoor_id|{}\nklv|{}\nmeta|{}\nplatformID|{}\ndeviceVersion|{}\nzf|{}\ncountry|{}\nuser|{}\nwk|{}\n",
                    bot.login_info.uuid, bot.login_info.protocol, bot.login_info.fhash, bot.login_info.mac, bot.login_info.requested_name, bot.login_info.hash2, bot.login_info.fz, bot.login_info.f, bot.login_info.player_age, bot.login_info.game_version, bot.login_info.lmode, bot.login_info.cbits, bot.login_info.rid, bot.login_info.gdpr, bot.login_info.hash, bot.login_info.category, bot.login_info.token, bot.login_info.total_playtime, bot.login_info.door_id, bot.login_info.klv, bot.login_info.meta, bot.login_info.platform_id, bot.login_info.device_version, bot.login_info.zf, bot.login_info.country, bot.login_info.user, bot.login_info.wk
                );
                bot.send_packet(peer, EPacketType::NetMessageGenericText, message);
            } else {
                let message = format!(
                    "protocol|{}\nltoken|{}\nplatformID|{}\n",
                    208, bot.token, "0,1,1"
                );
                bot.send_packet(peer, EPacketType::NetMessageGenericText, message);
            }
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
                variant_handler::handle(bot, peer, &tank_packet, &data[56..]);
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
