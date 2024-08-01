use std::sync::{Arc, Mutex};

use enet::{Peer, PeerID};
use spdlog::{info, warn};

use crate::bot::{disconnect, find_path, place, punch, send_packet, talk, walk};
use crate::types::e_packet_type::EPacketType;
use crate::types::tank_packet_type::TankPacketType;
use crate::utils::text_parse;
use crate::utils::variant::VariantList;

use super::Bot;

pub fn handle(bot_mutex: &Arc<Mutex<Bot>>, pkt: &TankPacketType, data: &[u8]) {
    let mut bot = bot_mutex.lock().unwrap();
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

            bot.state.is_redirect = true;
            bot.server.ip = parsed_server_data.get(0).unwrap().to_string();
            bot.server.port = port.to_string();
            bot.info.login_info.token = token.to_string();
            bot.info.login_info.user = user_id.to_string();
            bot.info.login_info.door_id = parsed_server_data.get(1).unwrap().to_string();
            bot.info.login_info.uuid = parsed_server_data.get(2).unwrap().to_string();
            let peer_id = bot.peer_id.unwrap();
            disconnect(peer_id);
        }
        "OnSuperMainStartAcceptLogonHrdxs47254722215a" => {
            let peer_id = bot.peer_id.unwrap();
            send_packet(
                peer_id,
                EPacketType::NetMessageGenericText,
                "action|enter_game\n".to_string(),
            );
            bot.state.is_redirect = false;
        }
        "OnCountryState" => {
            // I'm not sure why this is sent twice, but it is.
            let peer_id = bot.peer_id.unwrap();
            send_packet(
                peer_id,
                EPacketType::NetMessageGenericText,
                "action|getDRAnimations\n".to_string(),
            );
            send_packet(
                peer_id,
                EPacketType::NetMessageGenericText,
                "action|getDRAnimations\n".to_string(),
            );
        }
        "OnDialogRequest" => {
            let message = variant.get(1).unwrap().as_string();
            if message.contains("Gazette") {
                let peer_id = bot.peer_id.unwrap();
                send_packet(
                    peer_id,
                    EPacketType::NetMessageGenericText,
                    "action|dialog_return\ndialog_name|gazette\nbuttonClicked|banner\n".to_string(),
                );
            }
        }
        "OnSetBux" => {
            let bux = variant.get(1).unwrap().as_int32();
            bot.state.gems = bux;
        }
        "OnConsoleMessage" => {
            let message = variant.get(1).unwrap().as_string();
            info!("Received console message: {}", message);
        }
        "OnSetPos" => {
            let pos = variant.get(1).unwrap().as_vec2();
            info!("Received position: {:?}", pos);
            bot.position.x = pos.0;
            bot.position.y = pos.1;
            if bot.state.is_ingame {
                let peer_id = bot.peer_id.unwrap();
                place(&bot_mutex, peer_id, 0, -1, 9640);
            }
        }
        "ShowStartFTUEPopup" => {
            return;
        }
        "OnFtueButtonDataSet" => {
            let unknown_1 = variant.get(1).unwrap().as_int32();
            let current_progress = variant.get(2).unwrap().as_int32();
            let total_progress = variant.get(3).unwrap().as_int32();
            let info = variant.get(4).unwrap().as_string();
            info!(
                "Received FTUE button data set: {} {} {} {}",
                unknown_1, current_progress, total_progress, info
            );
        }
        "OnHideMenusRequest" => {
            warn!("Received OnHideMenusRequest");
        }
        "OnSpawn" => {
            let message = variant.get(1).unwrap().as_string();
            let data = text_parse::parse_and_store_as_map(&message);
            bot.state.is_ingame = true;
            bot.state.net_id = data.get("netID").unwrap().parse().unwrap();
        }
        "OnTalkBubble" => {
            let message = variant.get(2).unwrap().as_string();
            print!("Received talk bubble: {}", message);
            if message.contains("mate right") {
                let peer_id = bot.peer_id.unwrap();
                walk(&bot_mutex, peer_id, 1.0, 0.0, false);
            }
            if message.contains("mate left") {
                let peer_id = bot.peer_id.unwrap();
                walk(&bot_mutex, peer_id, -1.0, 0.0, false);
            }
            if message.contains("mate up") {
                let peer_id = bot.peer_id.unwrap();
                walk(&bot_mutex, peer_id, 0.0, -1.0, false);
            }
            if message.contains("mate down") {
                let peer_id = bot.peer_id.unwrap();
                walk(&bot_mutex, peer_id, 0.0, 1.0, false);
            }
            if message.contains("mate say") {
                let peer_id = bot.peer_id.unwrap();
                talk(peer_id, "Hello, world!");
            }
            if message.contains("mate punch") {
                let peer_id = bot.peer_id.unwrap();
                punch(&bot_mutex, peer_id, 0, 1);
            }
            if message.contains("mate findp") {
                let peer_id = bot.peer_id.unwrap();
                find_path(&bot_mutex, peer_id, 30, 5);
            }
        }
        "OnClearTutorialArrow" => {
            let v1 = variant.get(1).unwrap().as_string();

            println!("Received OnClearTutorialArrow: {} ", v1);
        }
        _ => {}
    }
}
