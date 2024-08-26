use crate::bot;
use crate::bot::{disconnect, is_inworld, send_packet};
use crate::types::epacket_type::EPacketType;
use crate::types::etank_packet_type::ETankPacketType;
use crate::types::player::Player;
use crate::types::tank_packet::TankPacket;
use crate::types::vector::Vector2;
use crate::utils::variant::VariantList;
use crate::utils::{self, textparse};
use paris::info;
use std::sync::Arc;

use super::Bot;

pub fn handle(bot: &Arc<Bot>, _: &TankPacket, data: &[u8]) {
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

            let mut state = bot.state.write();
            let mut server = bot.server.write();
            let mut info = bot.info.write();

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
            bot.state.write().is_redirecting = false;
        }
        "OnCountryState" => {
            // currently i don't know what is vector_y2 is calculated.
            let position = bot.position.read().clone();
            let mut pkt = TankPacket {
                _type: ETankPacketType::NetGamePacketState,
                flags: 4,
                vector_x: position.x,
                vector_y: position.y,
                ..Default::default()
            };
            bot::send_packet_raw(bot, &pkt);
            pkt.flags = 38;
            bot::send_packet_raw(bot, &pkt);
            pkt.flags = 34;
            bot::send_packet_raw(bot, &pkt);
            bot::send_packet_raw(bot, &pkt);
        }
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
            bot.state.write().gems = bux;
        }
        "OnConsoleMessage" => {
            let message = variant.get(1).unwrap().as_string();
            info!("Received console message: {}", message);
        }
        "OnSetPos" => {
            let pos = variant.get(1).unwrap().as_vec2();
            info!("Received position: {:?}", pos);
            let mut position = bot.position.write();
            position.x = pos.0;
            position.y = pos.1;
        }
        "SetHasGrowID" => {
            let growid = variant.get(2).unwrap().as_string();
            let mut info = bot.info.write();
            info.login_info.tank_id_name = growid;
            utils::config::save_token_to_bot(
                info.username.clone(),
                info.token.clone(),
                info.login_info.to_string(),
            );
        }
        "ShowStartFTUEPopup" => {}
        "OnFtueButtonDataSet" => {
            let unknown_1 = variant.get(1).unwrap().as_int32();
            let current_progress = variant.get(2).unwrap().as_int32();
            let total_progress = variant.get(3).unwrap().as_int32();
            let info = variant.get(4).unwrap().as_string();
            info!(
                "Received FTUE button data set: {} {} {} {}",
                unknown_1, current_progress, total_progress, info
            );

            let mut ftue = bot.ftue.write();
            ftue.current_progress = current_progress;
            ftue.total_progress = total_progress;
            ftue.info = info;
        }
        "OnSpawn" => {
            let message = variant.get(1).unwrap().as_string();
            let data = utils::textparse::parse_and_store_as_map(&message);
            if data.contains_key("type") {
                if data.get("type").unwrap() == "local" {
                    let mut state = bot.state.write();
                    state.is_ingame = true;
                    state.net_id = data.get("netID").unwrap().parse().unwrap();

                    send_packet(
                        bot,
                        EPacketType::NetMessageGenericText,
                        "action|getDRAnimations\n".to_string(),
                    );
                    return;
                }
            } else {
                let player = Player {
                    _type: data.get("type").unwrap_or(&"".to_string()).to_string(),
                    avatar: data.get("avatar").unwrap_or(&"".to_string()).to_string(),
                    net_id: data
                        .get("netID")
                        .unwrap_or(&"0".to_string())
                        .parse()
                        .unwrap_or(0),
                    online_id: data
                        .get("onlineID")
                        .unwrap_or(&"0".to_string())
                        .parse()
                        .unwrap_or("".to_string()),
                    e_id: data
                        .get("eid")
                        .unwrap_or(&"0".to_string())
                        .parse()
                        .unwrap_or("".to_string()),
                    ip: data.get("ip").unwrap_or(&"".to_string()).to_string(),
                    colrect: data.get("colrect").unwrap_or(&"".to_string()).to_string(),
                    title_icon: data.get("titleIcon").unwrap_or(&"".to_string()).to_string(),
                    mstate: data
                        .get("mstate")
                        .unwrap_or(&"0".to_string())
                        .parse()
                        .unwrap_or(0),
                    user_id: data
                        .get("userID")
                        .unwrap_or(&"0".to_string())
                        .parse()
                        .unwrap_or(0),
                    invis: data
                        .get("invis")
                        .unwrap_or(&"0".to_string())
                        .parse()
                        .unwrap_or(false),
                    name: data.get("name").unwrap_or(&"".to_string()).to_string(),
                    country: data.get("country").unwrap_or(&"".to_string()).to_string(),
                    position: {
                        if data.contains_key("posXY") {
                            let pos_xy = data.get("posXY").unwrap();
                            Vector2 {
                                x: pos_xy[..pos_xy.find("|").unwrap()].parse().unwrap_or(0.0),
                                y: pos_xy[pos_xy.find("|").unwrap() + 1..]
                                    .parse()
                                    .unwrap_or(0.0),
                            }
                        } else {
                            Vector2 { x: 0.0, y: 0.0 }
                        }
                    },
                };
                let mut players = bot.players.write();
                players.push(player);
            }
        }
        "OnRemove" => {
            let message = variant.get(1).unwrap().as_string();
            let data = utils::textparse::parse_and_store_as_map(&message);
            let net_id: u32 = data.get("netID").unwrap().parse().unwrap();

            let mut players = bot.players.write();
            players.retain(|player| player.net_id != net_id);
        }
        "OnTalkBubble" => {
            let message = variant.get(2).unwrap().as_string();
            info!("Received talk bubble message: {}", message);
        }
        "OnClearTutorialArrow" => {
            let v1 = variant.get(1).unwrap().as_string();
            info!("Received OnClearTutorialArrow: {} ", v1);
        }
        "OnRequestWorldSelectMenu" => {
            bot.world.write().reset();
            bot.players.write().clear();
        }
        _ => {}
    }
}
