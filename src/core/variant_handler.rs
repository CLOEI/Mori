use super::Bot;
use crate::core;
use crate::types::epacket_type::EPacketType;
use crate::types::player::Player;
use crate::types::tank_packet::TankPacket;
use crate::types::vector::Vector2;
use crate::utils::variant::VariantList;
use crate::utils::{self, textparse};
use std::sync::Arc;
use std::thread;

pub fn handle(bot: Arc<Bot>, _: &TankPacket, data: &[u8]) {
    let variant = VariantList::deserialize(&data).unwrap();
    let function_call: String = variant.get(0).unwrap().as_string();
    bot.log_info(format!("Received function call: {}", function_call).as_str());

    match function_call.as_str() {
        "OnSendToServer" => {
            let port = variant.get(1).unwrap().as_int32();
            let token = variant.get(2).unwrap().as_int32();
            let user_id = variant.get(3).unwrap().as_int32();
            let server_data = variant.get(4).unwrap().as_string();
            let parsed_server_data = textparse::parse_and_store_as_vec(&server_data);
            let aat = variant.get(5).unwrap().as_int32();

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
            info.login_info.aat = aat.to_string();
            bot.disconnect();
        }
        "OnSuperMainStartAcceptLogonHrdxs47254722215a" => {
            let server_hash = variant.get(1).unwrap().as_uint32();
            let item_database_loaded = {
                let item_database = bot.item_database.read().unwrap();
                item_database.loaded
            };

            if !item_database_loaded {
                match utils::proton::hash_file("items.dat") {
                    Ok(hash) => {
                        if hash == server_hash {
                            let mut item_database = bot.item_database.write().unwrap();
                            *item_database = gtitem_r::load_from_file("items.dat").unwrap();
                            bot.send_packet(
                                EPacketType::NetMessageGenericText,
                                "action|enter_game\n".to_string(),
                            );
                            let mut state = bot.state.lock().unwrap();
                            state.is_redirecting = false;
                            state.is_ingame = true;
                            state.is_allowed_to_warp = true;
                            return;
                        }
                    }
                    Err(_) => {}
                }

                bot.send_packet(
                    EPacketType::NetMessageGenericText,
                    "action|refresh_item_data\n".to_string(),
                );
                let item_database_clone = bot.item_database.clone();
                let bot_clone = Arc::clone(&bot);
                thread::spawn(move || {
                    loop {
                        let item_database_loaded = {
                            let item_database = item_database_clone.read().unwrap();
                            item_database.loaded
                        };
                        if item_database_loaded {
                            break;
                        }
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    bot_clone.send_packet(
                        EPacketType::NetMessageGenericText,
                        "action|enter_game\n".to_string(),
                    );
                    let mut state = bot_clone.state.lock().unwrap();
                    state.is_redirecting = false;
                    state.is_ingame = true;
                    state.is_allowed_to_warp = true;
                });
            } else {
                bot.send_packet(
                    EPacketType::NetMessageGenericText,
                    "action|enter_game\n".to_string(),
                );
                let mut state = bot.state.lock().unwrap();
                state.is_redirecting = false;
                state.is_ingame = true;
                state.is_allowed_to_warp = true;
            }
        }
        "OnCountryState" => {}
        "OnDialogRequest" => {
            let message = variant.get(1).unwrap().as_string();
            let cb = {
                let temp = bot.temporary_data.read().unwrap();
                temp.dialog_callback.clone()
            };

            if let Some(callback) = cb {
                callback(&bot);
            }

            bot.log_info(format!("Received dialog request: {}", message).as_str());
            if message.contains("Gazette") {
                bot.send_packet(
                    EPacketType::NetMessageGenericText,
                    "action|dialog_return\ndialog_name|gazette\nbuttonClicked|banner\n".to_string(),
                );
            }
            if message.contains("Trash") {
                let (item_id, amount) = {
                    let temp_data = bot.temporary_data.read().unwrap();
                    temp_data.trash
                };
                bot.send_packet(
                    EPacketType::NetMessageGenericText,
                    format!(
                        "action|dialog_return\ndialog_name|trash_item\nitemID|{}|\ncount|{}\n",
                        item_id, amount
                    ),
                );
            }
            if message.contains("Drop") {
                let (item_id, amount) = {
                    let temp_data = bot.temporary_data.read().unwrap();
                    temp_data.drop
                };
                bot.send_packet(
                    EPacketType::NetMessageGenericText,
                    format!(
                        "action|dialog_return\ndialog_name|drop_item\nitemID|{}|\ncount|{}\n",
                        item_id, amount
                    ),
                );
            }
        }
        "OnSetBux" => {
            let bux = variant.get(1).unwrap().as_int32();
            let mut state = bot.state.lock().unwrap();
            state.gems = bux;
        }
        "OnConsoleMessage" => {
            let message = variant.get(1).unwrap().as_string();
            bot.log_info(format!("Received console message: {}", message).as_str());
            if message.contains("wants to add you to")
                && message.contains("Wrench yourself to accept")
            {
                bot.send_packet(
                    EPacketType::NetMessageGenericText,
                    format!(
                        "action|wrench\n|netid|{}\n",
                        bot.state.lock().unwrap().net_id
                    ),
                );
            }
        }
        "OnSetPos" => {
            let pos = variant.get(1).unwrap().as_vec2();
            bot.log_info(format!("Received position: {:?}", pos).as_str());
            let mut position = bot.position.lock().unwrap();
            let mut temp = bot.temporary_data.write().unwrap();
            let (world_name, main_door_x, main_door_y) = {
                let world = bot.world.read().unwrap();
                let mut data = (0.0, 0.0);
                if let Some(item) = world.tiles.iter().find(|item| item.foreground_item_id == 6) {
                    data = (item.x as f32 * 32.0, item.y as f32 * 32.0);
                } else {
                    data = (0.0, 0.0);
                }
                (world.name.clone(), data.0, data.1)
            };

            let is_warp_to_id = world_name.contains("|");

            if !temp.entered_world
                && (pos.0 != main_door_x || pos.1 != main_door_y)
                && !is_warp_to_id
            {
                position.x = main_door_x;
                position.y = main_door_y;
                temp.entered_world = true;
                return;
            }

            position.x = pos.0;
            position.y = pos.1;
            temp.entered_world = true;
        }
        "SetHasGrowID" => {
            let growid = variant.get(2).unwrap().as_string();
            let mut info = bot.info.lock().unwrap();
            info.login_info.tank_id_name = growid;
            utils::config::save_token_to_bot(
                info.payload[0].clone(),
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
            bot.log_info(
                format!(
                    "Received FTUE button data set: {} {} {} {}",
                    unknown_1, current_progress, total_progress, info
                )
                .as_str(),
            );

            let mut ftue = bot.ftue.lock().unwrap();
            ftue.current_progress = current_progress;
            ftue.total_progress = total_progress;
            ftue.info = info;
        }
        "OnSpawn" => {
            let message = variant.get(1).unwrap().as_string();
            let data = textparse::parse_and_store_as_map(&message);
            if data.contains_key("type") {
                if data.get("type").unwrap() == "local" {
                    {
                        let mut state = bot.state.lock().unwrap();
                        state.net_id = data.get("netID").unwrap().parse().unwrap();
                        state.is_allowed_to_warp = true;
                    }
                    bot.send_packet(
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
                if player.invis {
                    bot.leave();
                }
                let mut players = bot.players.lock().unwrap();
                players.push(player);
            }
        }
        "OnRemove" => {
            let message = variant.get(1).unwrap().as_string();
            let data = textparse::parse_and_store_as_map(&message);
            let net_id: u32 = data.get("netID").unwrap().parse().unwrap();

            let mut players = bot.players.lock().unwrap();
            players.retain(|player| player.net_id != net_id);
        }
        "OnTalkBubble" => {
            let message = variant.get(2).unwrap().as_string();
            bot.log_info(format!("Received talk bubble message: {}", message).as_str());
        }
        "OnClearTutorialArrow" => {
            let v1 = variant.get(1).unwrap().as_string();
            bot.log_info(format!("Received OnClearTutorialArrow: {} ", v1).as_str());
        }
        "OnRequestWorldSelectMenu" => {
            bot.world.write().unwrap().reset();
            bot.players.lock().unwrap().clear();
        }
        "OnFailedToEnterWorld" => {
            bot.state.lock().unwrap().is_allowed_to_warp = true;
        }
        _ => {}
    }
}
