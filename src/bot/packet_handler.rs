use std::sync::{Arc, Mutex};

use crate::{bot::send_packet, types::epacket_type::EPacketType};

use super::Bot;

pub fn handle(bot: &Arc<Mutex<Bot>>, packet_type: EPacketType, data: &[u8]) {
    match packet_type {
        EPacketType::NetMessageServerHello => {
            let b = bot.lock().unwrap();
            if b.state.is_redirecting {
                let message = format!(
                    "UUIDToken|{}\nprotocol|{}\nfhash|{}\nmac|{}\nrequestedName|{}\nhash2|{}\nfz|{}\nf|{}\nplayer_age|{}\ngame_version|{}\nlmode|{}\ncbits|{}\nrid|{}\nGDPR|{}\nhash|{}\ncategory|{}\ntoken|{}\ntotal_playtime|{}\ndoor_id|{}\nklv|{}\nmeta|{}\nplatformID|{}\ndeviceVersion|{}\nzf|{}\ncountry|{}\nuser|{}\nwk|{}\n",
                    b.info.login_info.uuid, b.info.login_info.protocol, b.info.login_info.fhash, b.info.login_info.mac, b.info.login_info.requested_name, b.info.login_info.hash2, b.info.login_info.fz, b.info.login_info.f, b.info.login_info.player_age, b.info.login_info.game_version, b.info.login_info.lmode, b.info.login_info.cbits, b.info.login_info.rid, b.info.login_info.gdpr, b.info.login_info.hash, b.info.login_info.category, b.info.login_info.token, b.info.login_info.total_playtime, b.info.login_info.door_id, b.info.login_info.klv, b.info.login_info.meta, b.info.login_info.platform_id, b.info.login_info.device_version, b.info.login_info.zf, b.info.login_info.country, b.info.login_info.user, b.info.login_info.wk);
                send_packet(&bot, EPacketType::NetMessageGenericText, message);
            } else {
                let message = format!(
                    "protocol|{}\nltoken|{}\nplatformID|{}\n",
                    209, b.info.token, "0,1,1"
                );
                send_packet(&bot, EPacketType::NetMessageGenericText, message);
            }
        }
        EPacketType::NetMessageGenericText => {}
        EPacketType::NetMessageGameMessage => {}
        EPacketType::NetMessageGamePacket => {}
        EPacketType::NetMessageError => {}
        EPacketType::NetMessageTrack => {}
        EPacketType::NetMessageClientLogRequest => {}
        EPacketType::NetMessageClientLogResponse => {}
        EPacketType::NetMessageMax => {}
        _ => (),
    }
}
