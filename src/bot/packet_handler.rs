use std::{fs, sync::Arc};
use paris::{error, info, warn};
use regex::Regex;
use crate::{
    bot::{self, variant_handler},
    types::{
        epacket_type::EPacketType, etank_packet_type::ETankPacketType, tank_packet::TankPacket,
    },
    utils,
};
use super::{inventory::InventoryItem, send_packet_raw, Bot};

pub fn handle(bot: &Arc<Bot>, packet_type: EPacketType, data: &[u8]) {
    match packet_type {
        EPacketType::NetMessageServerHello => {
            let is_redirecting = bot.state.read().unwrap().is_redirecting;
            if is_redirecting {
                let message = {
                    let info = bot.info.read().unwrap();
                    format!(
                        "UUIDToken|{}\nprotocol|{}\nfhash|{}\nmac|{}\nrequestedName|{}\nhash2|{}\nfz|{}\nf|{}\nplayer_age|{}\ngame_version|{}\nlmode|{}\ncbits|{}\nrid|{}\nGDPR|{}\nhash|{}\ncategory|{}\ntoken|{}\ntotal_playtime|{}\ndoor_id|{}\nklv|{}\nmeta|{}\nplatformID|{}\ndeviceVersion|{}\nzf|{}\ncountry|{}\nuser|{}\nwk|{}\n",
                        info.login_info.uuid, info.login_info.protocol, info.login_info.fhash, info.login_info.mac, info.login_info.requested_name, info.login_info.hash2, info.login_info.fz, info.login_info.f, info.login_info.player_age, info.login_info.game_version, info.login_info.lmode, info.login_info.cbits, info.login_info.rid, info.login_info.gdpr, info.login_info.hash, info.login_info.category, info.login_info.token, info.login_info.total_playtime, info.login_info.door_id, info.login_info.klv, info.login_info.meta, info.login_info.platform_id, info.login_info.device_version, info.login_info.zf, info.login_info.country, info.login_info.user, info.login_info.wk
                    )
                };
                bot::send_packet(&bot, EPacketType::NetMessageGenericText, message);
            } else {
                let token = bot.info.read().unwrap().token.clone();
                let message = format!(
                    "protocol|{}\nltoken|{}\nplatformID|{}\n",
                    209, token, "0,1,1"
                );
                bot::send_packet(&bot, EPacketType::NetMessageGenericText, message);
            }
        }
        EPacketType::NetMessageGenericText => {}
        EPacketType::NetMessageGameMessage => {
            let message = String::from_utf8_lossy(&data);
            info!("Message: {}", message);

            if message.contains("logon_fail") {
                bot.state.write().unwrap().is_redirecting = false;
                bot::disconnect(bot);
            }
            if message.contains("currently banned") {
                let mut state = bot.state.write().unwrap();
                state.is_running = false;
                state.is_banned = true;
                bot::disconnect(bot);
            }
            if message.contains("Advanced Account Protection") {
                bot.state.write().unwrap().is_running = false;
                bot::disconnect(bot);
            }
            if message.contains("temporarily suspended") {
                bot.state.write().unwrap().is_running = false;
                bot::disconnect(bot);
            }
            if message.contains("has been suspended") {
                let mut state = bot.state.write().unwrap();
                state.is_running = false;
                state.is_banned = true;
                bot::disconnect(bot);
            }
            if message.contains("Growtopia is not quite ready for users") {
                let mut info = bot.info.write().unwrap();
                info.timeout = 60;
                bot::sleep(bot);
            }
            if message.contains("UPDATE REQUIRED") {
                let re = Regex::new(r"\$V(\d+\.\d+)").unwrap();
                if let Some(caps) = re.captures(&message) {
                    let version = caps.get(1).unwrap().as_str();
                    warn!("Update required: {}, updating...", version);
                    {
                        bot.info.write().unwrap().login_info.game_version = version.to_string();
                    }
                    utils::config::set_game_version(version.to_string());
                    let username = bot.info.read().unwrap().payload[0].clone();
                    utils::config::save_token_to_bot(username, "".to_string(), "".to_string());
                }
            }
        }
        EPacketType::NetMessageGamePacket => match bincode::deserialize::<TankPacket>(&data) {
            Ok(tank_packet) => {
                info!("Received: {:?}", tank_packet._type);
                match tank_packet._type {
                    ETankPacketType::NetGamePacketState => {
                        let mut players = bot.players.write().unwrap();
                        for player in players.iter_mut() {
                            if player.net_id == tank_packet.net_id {
                                player.position.x = tank_packet.vector_x;
                                player.position.y = tank_packet.vector_y;
                                break;
                            }
                        }
                    }
                    ETankPacketType::NetGamePacketCallFunction => {
                        variant_handler::handle(bot, &tank_packet, &data[56..]);
                    }
                    ETankPacketType::NetGamePacketPingRequest => {
                        let packet = TankPacket {
                            _type: ETankPacketType::NetGamePacketPingReply,
                            vector_x: 64.0,
                            vector_y: 64.0,
                            vector_x2: 1000.0,
                            vector_y2: 250.0,
                            value: tank_packet.value,
                            unk4: utils::proton::hash_string(&tank_packet.value.to_string()),
                            ..Default::default()
                        };

                        send_packet_raw(&bot, &packet);
                        info!("Replied to ping request");
                    }
                    ETankPacketType::NetGamePacketSendInventoryState => {
                        bot.inventory.write().unwrap().parse(&data[56..]);
                    }
                    ETankPacketType::NetGamePacketSendMapData => {
                        fs::write("world.dat", &data[56..]).unwrap();
                        {
                            let mut world = bot.world.write().unwrap();
                            world.parse(&data[56..]);
                        }
                        bot.astar.write().unwrap().update(bot);
                        bot::send_packet(
                            bot,
                            EPacketType::NetMessageGenericText,
                            "action|getDRAnimations\n".to_string(),
                        );
                    }
                    ETankPacketType::NetGamePacketTileChangeRequest => {
                        let should_update_inventory = {
                            let state = bot.state.read().unwrap();
                            state.net_id == tank_packet.net_id && tank_packet.value != 18
                        };

                        if should_update_inventory {
                            let mut remove_item = None;
                            {
                                let mut inventory = bot.inventory.write().unwrap();
                                if let Some(item) = inventory.items.get_mut(&(tank_packet.value as u16)) {
                                    item.amount -= 1;
                                    if item.amount == 0 || item.amount > 200 {
                                        remove_item = Some(tank_packet.value as u16);
                                    }
                                }
                            }
                            if let Some(item_id) = remove_item {
                                let mut inventory = bot.inventory.write().unwrap();
                                inventory.items.remove(&item_id);
                            }
                        }

                        {
                            let mut world = bot.world.write().unwrap();
                            if let Some(tile) = world.get_tile_mut(tank_packet.int_x as u32, tank_packet.int_y as u32) {
                                if tank_packet.value == 18 {
                                    if tile.foreground_item_id != 0 {
                                        tile.foreground_item_id = 0;
                                    } else {
                                        tile.background_item_id = 0;
                                    }
                                } else {
                                    if let Some(item) = bot.item_database.items.get(&tank_packet.value) {
                                        if item.action_type == 22 || item.action_type == 28 || item.action_type == 18 {
                                            tile.background_item_id = tank_packet.value as u16;
                                        } else {
                                            info!("TileChangeRequest: {:?}", tank_packet);
                                            tile.foreground_item_id = tank_packet.value as u16;
                                        }
                                    }
                                }
                            }
                        }

                        bot.astar.write().unwrap().update(bot);
                    }
                    ETankPacketType::NetGamePacketItemChangeObject => {
                        let mut world = bot.world.write().unwrap();
                        info!("ItemChangeObject: {:?}", tank_packet);

                        if tank_packet.net_id == u32::MAX {
                            let item = gtworld_r::DroppedItem {
                                id: tank_packet.value as u16,
                                x: tank_packet.vector_x.ceil(),
                                y: tank_packet.vector_y.ceil(),
                                count: tank_packet.unk6 as u8,
                                flags: tank_packet.unk1,
                                uid: world.dropped.last_dropped_item_uid + 1,
                            };

                            world.dropped.items.push(item);
                            world.dropped.last_dropped_item_uid += 1;
                            world.dropped.items_count += 1;
                            return;
                        } else if tank_packet.net_id == u32::MAX - 3 {
                            for obj in &mut world.dropped.items {
                                if obj.id == tank_packet.value as u16
                                    && obj.x == tank_packet.vector_x.ceil()
                                    && obj.y == tank_packet.vector_y.ceil()
                                {
                                    obj.count = tank_packet.unk6 as u8;
                                    break;
                                }
                            }
                        } else if tank_packet.net_id > 0 {
                            let mut remove_index = None;
                            for (i, obj) in world.dropped.items.iter().enumerate() {
                                if obj.uid == tank_packet.value {
                                    if tank_packet.net_id == bot.state.read().unwrap().net_id {
                                        if obj.id == 112 {
                                            bot.state.write().unwrap().gems += obj.count as i32;
                                        } else {
                                            let mut inventory = bot.inventory.write().unwrap();
                                            if let Some(item) = inventory.items.get_mut(&obj.id) {
                                                let temp = item.amount + obj.count as u16;
                                                item.amount = if temp > 200 { 200 } else { temp };
                                            } else {
                                                let item = InventoryItem {
                                                    id: obj.id,
                                                    amount: obj.count as u16,
                                                };
                                                inventory.items.insert(obj.id, item);
                                            }
                                        }
                                    }
                                    remove_index = Some(i);
                                    break;
                                }
                            }
                            if let Some(i) = remove_index {
                                world.dropped.items.remove(i);
                                world.dropped.items_count -= 1;
                            }
                        }
                    }
                    ETankPacketType::NetGamePacketSendTileTreeState => {
                        let mut world = bot.world.write().unwrap();
                        let tile = world.get_tile_mut(tank_packet.int_x as u32, tank_packet.int_y as u32).unwrap();
                        if let gtworld_r::TileType::Seed { ready_to_harvest, .. } = &mut tile.tile_type {
                            *ready_to_harvest = true;
                        }
                    }
                    _ => {}
                }
            }
            Err(..) => {
                error!("Failed to deserialize TankPacket: {:?}", data[0]);
            }
        },
        EPacketType::NetMessageClientLogRequest => {
            let message = String::from_utf8_lossy(&data);
            info!("Message: {}", message);
        }
        _ => (),
    }
}
