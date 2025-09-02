use byteorder::{ByteOrder, LittleEndian};
use crate::{variant_handler, Bot};
use crate::types::net_game_packet::{NetGamePacket, NetGamePacketData};
use crate::types::net_message::NetMessage;

pub fn handle(bot: &Bot, data: &[u8]) {
    let packet_id = LittleEndian::read_u32(&data[0..4]);
    let packet_type = NetMessage::from(packet_id);

    match packet_type {
        NetMessage::ServerHello => {
            let is_redirecting = {
                let state = bot.state.is_redirecting.lock().unwrap();
                *state
            };

            let login_info_lock = &bot.info.login_info.lock().unwrap();
            let login_info = login_info_lock.as_ref().unwrap();

            let data;

            if is_redirecting {
                data = format!(
                    "UUIDToken|{}\nprotocol|{}\nfhash|{}\nmac|{}\nrequestedName|{}\n\
        hash2|{}\nfz|{}\nf|{}\nplayer_age|{}\ngame_version|{}\nlmode|{}\n\
        cbits|{}\nrid|{}\nGDPR|{}\nhash|{}\ncategory|{}\ntoken|{}\n\
        total_playtime|{}\ndoor_id|{}\nklv|{}\nmeta|{}\nplatformID|{}\n\
        deviceVersion|{}\nzf|{}\ncountry|{}\nuser|{}\nwk|{}\naat|{}\n",
                    login_info.uuid,
                    login_info.protocol,
                    login_info.fhash,
                    login_info.mac,
                    login_info.requested_name,
                    login_info.hash2,
                    login_info.fz,
                    login_info.f,
                    login_info.player_age,
                    login_info.game_version,
                    login_info.lmode,
                    login_info.cbits,
                    login_info.rid,
                    login_info.gdpr,
                    login_info.hash,
                    login_info.category,
                    login_info.token,
                    login_info.total_play_time,
                    login_info.door_id,
                    login_info.klv,
                    login_info.meta,
                    login_info.platform_id,
                    login_info.device_version,
                    login_info.zf,
                    login_info.country,
                    login_info.user,
                    login_info.wk,
                    login_info.aat,
                );

                {
                    let mut redirecting = bot.state.is_redirecting.lock().unwrap();
                    *redirecting = false;
                }

            } else {
                data = format!(
                    "protocol|{}\nltoken|{}\nplatformID|{}\n",
                    login_info.protocol,
                    login_info.ltoken,
                    login_info.platform_id
                );
            }
            bot.send_packet(NetMessage::GenericText, data);
        },
        NetMessage::GameMessage => {
            let message = String::from_utf8_lossy(&data[4..]).to_string();
            println!("GameMessage: {}", message);
        },
        NetMessage::GamePacket => {
            let parsed = NetGamePacketData::from_bytes(&data[4..]).expect("Failed to parse NetGamePacketData");
            println!("GamePacket: {:?}", parsed._type);
            match parsed._type {
                NetGamePacket::CallFunction => {
                    variant_handler::handle(bot, &data[60..]);
                }
                _ => {}
            }
        }
        _ => {}
    }
}