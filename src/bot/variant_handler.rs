use enet::Peer;
use spdlog::info;

use crate::types::e_packet_type::{self, EPacketType};
use crate::types::tank_packet_type::TankPacketType;
use crate::utils::{text_parse::parse_and_store, variant::VariantList};

use super::Bot;

pub fn handle(bot: &mut Bot, peer: &mut Peer<()>, pkt: &TankPacketType, data: &[u8]) {
    let variant = VariantList::deserialize(&data).unwrap();
    let function_call: String = variant.get(0).unwrap().as_string();
    info!("Received function call: {}", function_call);

    match function_call.as_str() {
        "OnSendToServer" => {
            let port = variant.get(1).unwrap().as_int32();
            let token = variant.get(2).unwrap().as_int32();
            let user_id = variant.get(3).unwrap().as_int32();
            let server_data = variant.get(4).unwrap().as_string();
            let username = variant.get(6).unwrap().as_string();
            let parsed_server_data = parse_and_store(&server_data);

            bot.is_redirect = true;
            bot.username = username;
            bot.server.ip = parsed_server_data.get(0).unwrap().to_string();
            bot.server.port = port.to_string();
            bot.login_info.token = token.to_string();
            bot.login_info.user = user_id.to_string();
            bot.login_info.door_id = parsed_server_data.get(1).unwrap().to_string();
            bot.login_info.uuid = parsed_server_data.get(2).unwrap().to_string();
            bot.disconnect(peer);
        }
        "OnSuperMainStartAcceptLogonHrdxs47254722215a" => bot.send_packet(
            peer,
            EPacketType::NetMessageGenericText,
            "action|enter_game\n".to_string(),
        ),
        "OnCountryState" => {
            // I'm not sure why this is sent twice, but it is.
            bot.send_packet(
                peer,
                EPacketType::NetMessageGenericText,
                "action|getDRAnimations\n".to_string(),
            );
            bot.send_packet(
                peer,
                EPacketType::NetMessageGenericText,
                "action|getDRAnimations\n".to_string(),
            );
        }
        _ => {}
    }
}
