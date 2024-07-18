use enet::Peer;
use spdlog::info;

use crate::types::e_packet_type::EPacketType;
use crate::types::tank_packet_type::TankPacketType;
use crate::utils::text_parse;
use crate::utils::variant::VariantList;

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
            let parsed_server_data = text_parse::parse_and_store_as_vec(&server_data);

            // if bot.display_name.is_empty() {
            //     bot.username = variant.get(6).unwrap().as_string();
            //     error!("Username: {}", bot.username);
            // }

            bot.is_redirect = true;
            bot.server.ip = parsed_server_data.get(0).unwrap().to_string();
            bot.server.port = port.to_string();
            bot.login_info.token = token.to_string();
            bot.login_info.user = user_id.to_string();
            bot.login_info.door_id = parsed_server_data.get(1).unwrap().to_string();
            bot.login_info.uuid = parsed_server_data.get(2).unwrap().to_string();
            bot.disconnect(peer);
        }
        "OnSuperMainStartAcceptLogonHrdxs47254722215a" => {
            bot.send_packet(
                peer,
                EPacketType::NetMessageGenericText,
                "action|enter_game\n".to_string(),
            );
            bot.is_redirect = false;
        }
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
        "OnDialogRequest" => {
            let message = variant.get(1).unwrap().as_string();
            if message.contains("Gazette") {
                bot.send_packet(
                    peer,
                    EPacketType::NetMessageGenericText,
                    "action|dialog_return\ndialog_name|gazette\nbuttonClicked|banner\n".to_string(),
                );
            }
        }
        "OnSetBux" => {
            let bux = variant.get(1).unwrap().as_int32();
            bot.gems = bux;
        }
        "OnConsoleMessage" => {
            let message = variant.get(1).unwrap().as_string();
            info!("Received console message: {}", message);
        }
        "OnSetPos" => {
            let pos = variant.get(1).unwrap().as_vec2();
            info!("Received position: {:?}", pos);
            bot.pos_x = pos.0;
            bot.pos_y = pos.1;
        }
        "OnSpawn" => {
            let message = variant.get(1).unwrap().as_string();
            let data = text_parse::parse_and_store_as_map(&message);
            bot.is_ingame = true;
            bot.net_id = data.get("netID").unwrap().parse().unwrap();
        }
        "OnTalkBubble" => {
            let message = variant.get(2).unwrap().as_string();
            print!("Received talk bubble: {}", message);
            if message.contains("mate right") {
                bot.walk(peer, 1.0, 0.0, false);
            }
            if message.contains("mate left") {
                bot.walk(peer, -1.0, 0.0, false);
            }
            if message.contains("mate up") {
                bot.walk(peer, 0.0, -1.0, false);
            }
            if message.contains("mate down") {
                bot.walk(peer, 0.0, 1.0, false);
            }
            if message.contains("mate say") {
                bot.talk(peer, "Hello, world!");
            }
            if message.contains("mate punch") {
                bot.punch(peer, 0, 1);
            }
            if message.contains("mate findp") {
                bot.find_path(peer, 0, 0);
            }
        }
        _ => {}
    }
}
