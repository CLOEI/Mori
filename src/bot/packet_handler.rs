use std::sync::Arc;

use paris::{error, info, warn};

use crate::{
    bot::{disconnect, send_packet, variant_handler},
    types::{
        epacket_type::EPacketType, etank_packet_type::ETankPacketType, tank_packet::TankPacket,
    },
    utils::mapping,
};

use super::{send_packet_raw, Bot};

pub fn handle(bot: &Arc<Bot>, packet_type: EPacketType, data: &[u8]) {
    match packet_type {
        EPacketType::NetMessageServerHello => {
            if bot.state.lock().unwrap().is_redirecting {
                let info = bot.info.lock().unwrap();
                let message = format!(
                    "UUIDToken|{}\nprotocol|{}\nfhash|{}\nmac|{}\nrequestedName|{}\nhash2|{}\nfz|{}\nf|{}\nplayer_age|{}\ngame_version|{}\nlmode|{}\ncbits|{}\nrid|{}\nGDPR|{}\nhash|{}\ncategory|{}\ntoken|{}\ntotal_playtime|{}\ndoor_id|{}\nklv|{}\nmeta|{}\nplatformID|{}\ndeviceVersion|{}\nzf|{}\ncountry|{}\nuser|{}\nwk|{}\n",
                    info.login_info.uuid, info.login_info.protocol, info.login_info.fhash, info.login_info.mac, info.login_info.requested_name, info.login_info.hash2, info.login_info.fz, info.login_info.f, info.login_info.player_age, info.login_info.game_version, info.login_info.lmode, info.login_info.cbits, info.login_info.rid, info.login_info.gdpr, info.login_info.hash, info.login_info.category, info.login_info.token, info.login_info.total_playtime, info.login_info.door_id, info.login_info.klv, info.login_info.meta, info.login_info.platform_id, info.login_info.device_version, info.login_info.zf, info.login_info.country, info.login_info.user, info.login_info.wk);
                send_packet(&bot, EPacketType::NetMessageGenericText, message);
            } else {
                let message = format!(
                    "protocol|{}\nltoken|{}\nplatformID|{}\n",
                    209,
                    bot.info.lock().unwrap().token,
                    "0,1,1"
                );
                send_packet(&bot, EPacketType::NetMessageGenericText, message);
            }
        }
        EPacketType::NetMessageGenericText => {}
        EPacketType::NetMessageGameMessage => {
            let message = String::from_utf8_lossy(&data);
            info!("Message: {}", message);

            if message.contains("logon_fail") {
                bot.state.lock().unwrap().is_redirecting = false;
                disconnect(bot);
            }
            if message.contains("currently banned") {
                {
                    let mut state = bot.state.lock().unwrap();
                    state.is_running = false;
                    state.is_banned = true;
                }
                disconnect(bot);
            }
            if message.contains("Advanced Account Protection") {
                bot.state.lock().unwrap().is_running = false;
                disconnect(bot);
            }
            if message.contains("temporarily suspended") {
                bot.state.lock().unwrap().is_running = false;
                disconnect(bot);
            }
        }
        EPacketType::NetMessageGamePacket => match bincode::deserialize::<TankPacket>(&data) {
            Ok(tank_packet) => match tank_packet._type {
                ETankPacketType::NetGamePacketCallFunction => {
                    variant_handler::handle(bot, &tank_packet, &data[56..]);
                }
                ETankPacketType::NetGamePacketPingRequest => {
                    let packet = TankPacket {
                        _type: ETankPacketType::NetGamePacketPingReply,
                        vector_x: 64.0,
                        vector_y: 64.0,
                        value: tank_packet.value + 5000,
                        ..Default::default()
                    };

                    send_packet_raw(&bot, &packet);
                }
                _ => {}
            },
            Err(..) => {
                error!("Failed to deserialize TankPacket: {:?}", data[0]);
            }
        },
        _ => (),
    }
}
