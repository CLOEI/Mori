use paris::info;
use std::sync::Arc;

use crate::bot::{disconnect, send_packet};
use crate::types::epacket_type::EPacketType;
use crate::types::player::Player;
use crate::types::tank_packet::TankPacket;
use crate::types::vector::Vector2;
use crate::utils::variant::VariantList;
use crate::utils::{self, textparse};

use super::Bot;

pub fn handle(bot: &Arc<Bot>, pkt: &TankPacket, data: &[u8]) {
    let variant = VariantList::deserialize(&data).unwrap();
    let function_call: String = variant.get(0).unwrap().as_string();
    info!("Received function call: {}", function_call);

    match function_call.as_str() {
        "OnSendToServer" => {
            let port = variant.get(1).unwrap().as_int32();
            let token = variant.get(2).unwrap().as_int32();
            let user_id = variant.get(3).unwrap().as_int32();
            let server_data = variant.get(4).unwrap().as_string();
            let parsed_server_data = textparse::parse_and_store_as_vec(&server_data);

            let mut state = bot.state.lock().unwrap();
            let mut server = bot.server.lock().unwrap();
            let mut info = bot.info.lock().unwrap();

            state.is_redirecting = true;
            server.ip = parsed_server_data.get(0).unwrap().to_string();
            server.port = port as u16;
            info.login_info.token = token.to_string();
            info.login_info.user = user_id.to_string();
            info.login_info.door_id = parsed_server_data.get(1).unwrap().to_string();
            info.login_info.uuid = parsed_server_data.get(2).unwrap().to_string();
            disconnect(bot);
        }
        "OnSuperMainStartAcceptLogonHrdxs47254722215a" => {
            send_packet(
                bot,
                EPacketType::NetMessageGenericText,
                "action|enter_game\n".to_string(),
            );
            bot.state.lock().unwrap().is_redirecting = false;
        }
        "OnCountryState" => {}
        "OnDialogRequest" => {
            let message = variant.get(1).unwrap().as_string();
            if message.contains("Gazette") {
                send_packet(
                    bot,
                    EPacketType::NetMessageGenericText,
                    "action|dialog_return\ndialog_name|gazette\nbuttonClicked|banner\n".to_string(),
                );
            }
        }
        "OnSetBux" => {
            let bux = variant.get(1).unwrap().as_int32();
            bot.state.lock().unwrap().gems = bux;
        }
        "OnConsoleMessage" => {
            let message = variant.get(1).unwrap().as_string();
            info!("Received console message: {}", message);
        }
        "OnSetPos" => {
            let pos = variant.get(1).unwrap().as_vec2();
            info!("Received position: {:?}", pos);
            let mut position = bot.position.lock().unwrap();
            position.x = pos.0;
            position.y = pos.1;
        }
        "SetHasGrowID" => {
            let growid = variant.get(2).unwrap().as_string();
            let mut info = bot.info.lock().unwrap();
            info.login_info.tank_id_name = growid;
            utils::config::save_token_to_bot(
                info.username.clone(),
                info.token.clone(),
                info.login_info.to_string(),
            );
        }
        "ShowStartFTUEPopup" => {}
        "OnFtueButtonDataSet" => {}
        "OnSpawn" => {
            let message = variant.get(1).unwrap().as_string();
            let data = utils::textparse::parse_and_store_as_map(&message);
            if data.contains_key("type") {
                if data.get("type").unwrap() == "local" {
                    let mut state = bot.state.lock().unwrap();
                    state.is_ingame = true;
                    state.net_id = data.get("netID").unwrap().parse().unwrap();
                    return;
                }
            } else {
                let player = Player {
                    _type: data.get("type").unwrap().to_string(),
                    avatar: data.get("avatar").unwrap().to_string(),
                    net_id: data.get("netID").unwrap().parse().unwrap(),
                    online_id: data.get("onlineID").unwrap().parse().unwrap(),
                    e_id: data.get("eid").unwrap().parse().unwrap(),
                    ip: data.get("ip").unwrap().to_string(),
                    colrect: data.get("colrect").unwrap().to_string(),
                    title_icon: data.get("titleIcon").unwrap().to_string(),
                    mstate: data.get("mstate").unwrap().parse().unwrap(),
                    user_id: data.get("userID").unwrap().parse().unwrap(),
                    invis: data.get("invisible").unwrap().parse().unwrap(),
                    name: data.get("name").unwrap().to_string(),
                    country: data.get("country").unwrap().to_string(),
                    position: {
                        if data.contains_key("posXY") {
                            let pos_xy = data.get("posXY").unwrap();
                            Vector2 {
                                x: pos_xy[..pos_xy.find("|").unwrap()].parse().unwrap(),
                                y: pos_xy[pos_xy.find("|").unwrap() + 1..].parse().unwrap(),
                            }
                        } else {
                            Vector2 { x: 0.0, y: 0.0 }
                        }
                    },
                };
                let mut players = bot.players.lock().unwrap();
                players.push(player);
            }
        }
        "OnRemove" => {
            let message = variant.get(1).unwrap().as_string();
            let data = utils::textparse::parse_and_store_as_map(&message);
            let net_id: u32 = data.get("netID").unwrap().parse().unwrap();

            let mut players = bot.players.lock().unwrap();
            players.retain(|player| player.net_id != net_id);
        }
        "OnTalkBubble" => {
            let message = variant.get(2).unwrap().as_string();
            info!("Received talk bubble message: {}", message);
        }
        "OnClearTutorialArrow" => {}
        "OnRequestWorldSelectMenu" => {
            bot.world.lock().unwrap().reset();
            bot.players.lock().unwrap().clear();
        }
        _ => {}
    }
}
