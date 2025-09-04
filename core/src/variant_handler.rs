use crate::types::net_message::NetMessage;
use crate::types::player::Player;
use crate::utils::proton::HashMode;
use crate::utils::variant::VariantList;
use crate::{Bot, utils};
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::Ordering;

pub fn handle(bot: &Bot, data: &[u8]) {
    let variant = VariantList::deserialize(&data).expect("Failed to deserialize variant list");
    let function_call: String = variant.get(0).unwrap().as_string();

    println!("Function call: {}", function_call);

    match function_call.as_str() {
        "OnSendToServer" => {
            let port = variant.get(1).unwrap().as_int32();
            let token = variant.get(2).unwrap().as_int32();
            let user_id = variant.get(3).unwrap().as_int32();
            let server_data = variant.get(4).unwrap().as_string();
            let parsed_server_data: Vec<String> = server_data
                .split('|')
                .map(|s| s.trim_end().to_string())
                .collect();
            let aat = variant.get(5).unwrap().as_int32();

            let mut server_data_lock = bot.info.server_data.lock().unwrap();
            let server_data = server_data_lock.as_mut().unwrap();

            server_data.server = parsed_server_data[0].clone();
            server_data.port = port as u16;

            let mut is_redirecting_lock = bot.is_redirecting.lock().unwrap();
            *is_redirecting_lock = true;

            let mut login_info_lock = bot.info.login_info.lock().unwrap();
            let login_info = login_info_lock.as_mut().unwrap();

            login_info.token = token.to_string();
            login_info.user = user_id.to_string();
            login_info.door_id = parsed_server_data[1].clone();
            login_info.uuid = parsed_server_data[2].clone();
            login_info.aat = aat.to_string();

            bot.disconnect()
        }
        "OnSuperMainStartAcceptLogonHrdxs47254722215a" => {
            let server_hash = variant.get(1).unwrap().as_uint32();

            match fs::read("items.dat") {
                Ok(data) => {
                    let hash = utils::proton::hash(
                        data.as_slice(),
                        HashMode::FixedLength(data.len() as i32),
                    ) as u32;

                    if hash == server_hash {
                        bot.send_packet(
                            NetMessage::GenericText,
                            "action|enter_game\n".to_string().as_bytes(),
                            None,
                            true,
                        );
                        let mut is_redirecting_lock = bot.is_redirecting.lock().unwrap();
                        *is_redirecting_lock = false;
                        let item_database = gtitem_r::load_from_file("items.dat")
                            .expect("Failed to load items.dat");
                        let mut item_database_lock = bot.item_database.write().unwrap();
                        *item_database_lock = item_database;
                        return;
                    }
                }
                Err(_) => {
                    println!("Fetching server items.dat...");
                }
            }

            bot.send_packet(
                NetMessage::GenericText,
                "action|refresh_item_data\n".to_string().as_bytes(),
                None,
                true,
            );
        }
        "OnSetPos" => {
            let pos = variant.get(1).unwrap().as_vec2();
            let mut position_lock = bot.position.write().unwrap();
            *position_lock = pos;
        }
        "OnTalkBubble" => {
            let message = variant.get(2).unwrap().as_string();
            println!("[TALK] {}", message);
        }
        "OnConsoleMessage" => {
            let message = variant.get(1).unwrap().as_string();
            println!("[CONSOLE] {}", message);
        }
        "OnSetBux" => {
            let gems = variant.get(1).unwrap().as_int32();
            let current = bot.gems.load(Ordering::SeqCst);
            bot.gems.store(current + gems, Ordering::SeqCst);
        }
        "SetHasGrowID" => {
            let growid = variant.get(2).unwrap().as_string();
            let mut login_info_lock = bot.info.login_info.lock().unwrap();
            let login_info = login_info_lock.as_mut().unwrap();
            login_info.tank_id_name = growid;
        }
        "OnRemove" => {
            let message = variant.get(1).unwrap().as_string();
            let data = parse_and_store_as_map(&message);
            let net_id: u32 = data["netID"].parse().unwrap();

            let mut players = bot.world.players.lock().unwrap();
            players.retain(|p| p.net_id != net_id);
        }
        "OnSpawn" => {
            let message = variant.get(1).unwrap().as_string();
            let data = parse_and_store_as_map(&message);

            if data.contains_key("type") {
                let mut net_id_lock = bot.net_id.lock().unwrap();
                *net_id_lock = data
                    .get("netID")
                    .unwrap()
                    .parse()
                    .expect("Failed to parse netid");
            } else {
                let player = Player {
                    _type: data["type"].clone(),
                    avatar: data["avatar"].clone(),
                    net_id: data["netID"].parse().expect("Failed to parse netid"),
                    online_id: data["onlineID"].parse().expect("Failed to parse onlineid"),
                    e_id: data["eid"].parse().expect("Failed to parse eid"),
                    ip: data["ip"].clone(),
                    col_rect: data["colrect"].clone(),
                    title_icon: data["titleIcon"].clone(),
                    m_state: data["mstate"].parse().expect("Failed to parse mstate"),
                    user_id: data["userID"].parse().expect("Failed to parse userid"),
                    invisible: data["invisible"]
                        .parse()
                        .expect("Failed to parse invisible"),
                    name: data["name"].clone(),
                    country: data["country"].clone(),
                    position: {
                        if data.contains_key("posXY") {
                            let pos_xy = data
                                .get("posXY")
                                .unwrap()
                                .split('|')
                                .map(|s| {
                                    s.trim().parse().expect("Fail to parse player coordinates")
                                })
                                .collect::<Vec<f32>>();
                            (pos_xy[0], pos_xy[1])
                        } else {
                            (0.0, 0.0)
                        }
                    },
                };

                let mut players = bot.world.players.lock().unwrap();
                players.push(player);
            }
        }
        "OnDialogRequest" => {
            let message = variant.get(1).unwrap().as_string();
            let cb = {
                let dialog_callback = bot.temporary_data.dialog_callback.lock().unwrap();
                dialog_callback.clone()
            };

            if let Some(cb) = cb {
                cb(bot);
            }

            if message.contains("Gazette") {
                bot.send_packet(
                    NetMessage::GenericText,
                    "action|dialog_return\ndialog_name|gazette\nbuttonClicked|banner\n".to_string().as_bytes(),
                    None,
                    true,
                );
            }
        }
        _ => {}
    }
}

fn parse_and_store_as_map(input: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in input.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 2 {
            let key = parts[0].to_string();
            let value = parts[1..].join("|");
            map.insert(key, value);
        }
    }
    map
}
