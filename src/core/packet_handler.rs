use super::{inventory::InventoryItem, Bot};
use crate::{
    core::variant_handler,
    types::{
        epacket_type::EPacketType, etank_packet_type::ETankPacketType, tank_packet::TankPacket,
    },
    utils,
};
use flate2::read::ZlibDecoder;
use gtworld_r::TileType;
use paris::error;
use regex::Regex;
use std::io::{Cursor, Read};
use std::time::Instant;
use std::{fs, sync::Arc};

pub fn handle(bot: Arc<Bot>, packet_type: EPacketType, data: &[u8]) {
    match packet_type {
        EPacketType::NetMessageServerHello => {
            let is_redirecting = bot.state.lock().unwrap().is_redirecting;
            if is_redirecting {
                let message = {
                    let info = bot.info.lock().unwrap();
                    format!(
                        "UUIDToken|{}\nprotocol|{}\nfhash|{}\nmac|{}\nrequestedName|{}\nhash2|{}\nfz|{}\nf|{}\nplayer_age|{}\ngame_version|{}\nlmode|{}\ncbits|{}\nrid|{}\nGDPR|{}\nhash|{}\ncategory|{}\ntoken|{}\ntotal_playtime|{}\ndoor_id|{}\nklv|{}\nmeta|{}\nplatformID|{}\ndeviceVersion|{}\nzf|{}\ncountry|{}\nuser|{}\nwk|{}\naat|{}\n",
                        info.login_info.uuid, 211, info.login_info.fhash, info.login_info.mac, info.login_info.requested_name, info.login_info.hash2, info.login_info.fz, info.login_info.f, info.login_info.player_age, info.login_info.game_version, info.login_info.lmode, info.login_info.cbits, info.login_info.rid, info.login_info.gdpr, info.login_info.hash, info.login_info.category, info.login_info.token, info.login_info.total_playtime, info.login_info.door_id, info.login_info.klv, info.login_info.meta, info.login_info.platform_id, info.login_info.device_version, info.login_info.zf, info.login_info.country, info.login_info.user, info.login_info.wk, info.login_info.aat
                    )
                };
                bot.send_packet(EPacketType::NetMessageGenericText, message);
            } else {
                let token = bot.info.lock().unwrap().token.clone();
                let message = {
                    let info = bot.info.lock().unwrap();
                    format!(
                        "protocol|{}\nltoken|{}\nplatformID|{}\n",
                        info.login_info.protocol, token, "0,1,1"
                    )
                };
                bot.send_packet(EPacketType::NetMessageGenericText, message);
            }
        }
        EPacketType::NetMessageGenericText => {}
        EPacketType::NetMessageGameMessage => {
            let message = String::from_utf8_lossy(&data);
            bot.log_info(format!("Message: {}", message).as_str());

            if message.contains("logon_fail") {
                {
                    let mut state = bot.state.lock().unwrap();
                    state.is_redirecting = false;
                }
                bot.disconnect();
            }
            if message.contains("currently banned") {
                let mut state = bot.state.lock().unwrap();
                state.is_running = false;
                state.is_banned = true;
                bot.disconnect();
            }
            if message.contains("Advanced Account Protection") {
                {
                    let mut state = bot.state.lock().unwrap();
                    state.is_running = false;
                }
                bot.disconnect();
            }
            if message.contains("temporarily suspended") {
                {
                    let mut state = bot.state.lock().unwrap();
                    state.is_running = false;
                }
                bot.disconnect();
            }
            if message.contains("has been suspended") {
                let mut state = bot.state.lock().unwrap();
                state.is_running = false;
                state.is_banned = true;
                bot.disconnect();
            }
            if message.contains("Growtopia is not quite ready for users") {
                let mut temp = bot.temporary_data.write().unwrap();
                temp.timeout = 60;
                bot.sleep();
            }
            if message.contains("UPDATE REQUIRED") {
                let re = Regex::new(r"\$V(\d+\.\d+)").unwrap();
                if let Some(caps) = re.captures(&message) {
                    let version = caps.get(1).unwrap().as_str();
                    bot.log_warn(format!("Update required: {}, updating...", version).as_str());
                    {
                        let mut info = bot.info.lock().unwrap();
                        info.login_info.game_version = version.to_string();
                    }
                    utils::config::set_game_version(version.to_string());
                    let username = bot.info.lock().unwrap().payload[0].clone();
                    utils::config::save_token_to_bot(username, "".to_string(), "".to_string());
                }
            }
        }
        EPacketType::NetMessageGamePacket => match bincode::deserialize::<TankPacket>(&data) {
            Ok(tank_packet) => {
                bot.log_info(format!("Received: {:?}", tank_packet._type).as_str());
                match tank_packet._type {
                    ETankPacketType::NetGamePacketDisconnect => {
                        bot.disconnect_now();
                    }
                    ETankPacketType::NetGamePacketState => {
                        let mut players = bot.players.lock().unwrap();
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
                            value: tank_packet.value + 5000,
                            ..Default::default()
                        };

                        bot.send_packet_raw(&packet);
                        bot.log_info("Replied to ping request");
                    }
                    ETankPacketType::NetGamePacketSendInventoryState => {
                        bot.inventory.lock().unwrap().parse(&data[56..]);
                    }
                    ETankPacketType::NetGamePacketSendMapData => {
                        fs::write("world.dat", &data[56..]).unwrap();
                        {
                            let mut world = bot.world.write().unwrap();
                            world.parse(&data[56..]);
                        }
                        bot.players.lock().unwrap().clear();
                        bot.astar.lock().unwrap().update(&bot);
                        bot.send_packet(
                            EPacketType::NetMessageGenericText,
                            "action|getDRAnimations\n".to_string(),
                        );
                    }
                    ETankPacketType::NetGamePacketTileChangeRequest => {
                        let should_update_inventory = {
                            let state = bot.state.lock().unwrap();
                            state.net_id == tank_packet.net_id && tank_packet.value != 18
                        };

                        if should_update_inventory {
                            let mut remove_item = None;
                            {
                                let mut inventory = bot.inventory.lock().unwrap();
                                if let Some(item) =
                                    inventory.items.get_mut(&(tank_packet.value as u16))
                                {
                                    item.amount -= 1;
                                    if item.amount == 0 || item.amount > 200 {
                                        remove_item = Some(tank_packet.value as u16);
                                    }
                                }
                            }
                            if let Some(item_id) = remove_item {
                                let mut inventory = bot.inventory.lock().unwrap();
                                inventory.items.remove(&item_id);
                            }
                        }

                        {
                            let mut world = bot.world.write().unwrap();
                            if let Some(tile) = world
                                .get_tile_mut(tank_packet.int_x as u32, tank_packet.int_y as u32)
                            {
                                if tank_packet.value == 18 {
                                    if tile.foreground_item_id != 0 {
                                        tile.foreground_item_id = 0;
                                    } else {
                                        tile.background_item_id = 0;
                                    }
                                } else {
                                    if let Some(item) = bot
                                        .item_database
                                        .read()
                                        .unwrap()
                                        .items
                                        .get(&tank_packet.value)
                                    {
                                        if item.action_type == 22
                                            || item.action_type == 28
                                            || item.action_type == 18
                                        {
                                            tile.background_item_id = tank_packet.value as u16;
                                        } else {
                                            tile.foreground_item_id = tank_packet.value as u16;
                                            let item = bot
                                                .item_database
                                                .read()
                                                .unwrap()
                                                .get_item(&tank_packet.value)
                                                .unwrap();
                                            if item.id % 2 != 0 {
                                                tile.tile_type = TileType::Seed {
                                                    ready_to_harvest: false,
                                                    time_passed: 0,
                                                    item_on_tree: 0,
                                                    elapsed: Instant::now().elapsed()
                                                };
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        bot.astar.lock().unwrap().update(&bot);
                    }
                    ETankPacketType::NetGamePacketItemChangeObject => {
                        let mut world = bot.world.write().unwrap();
                        bot.log_info(format!("ItemChangeObject: {:?}", tank_packet).as_str());

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
                                    if tank_packet.net_id == bot.state.lock().unwrap().net_id {
                                        if obj.id == 112 {
                                            bot.state.lock().unwrap().gems += obj.count as i32;
                                        } else {
                                            let mut inventory = bot.inventory.lock().unwrap();
                                            if let Some(item) = inventory.items.get_mut(&obj.id) {
                                                let temp = item.amount + obj.count;
                                                item.amount = if temp > 200 { 200 } else { temp };
                                            } else {
                                                let item = InventoryItem {
                                                    id: obj.id,
                                                    amount: obj.count,
                                                    flag: 0,
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
                        let tile = world
                            .get_tile_mut(tank_packet.int_x as u32, tank_packet.int_y as u32)
                            .unwrap();
                        tile.foreground_item_id = 0;
                        tile.tile_type = TileType::Basic;
                    }
                    ETankPacketType::NetGamePacketModifyItemInventory => {
                        let mut inventory = bot.inventory.lock().unwrap();
                        if let Some(item) = inventory.items.get_mut(&(tank_packet.value as u16)) {
                            item.amount -= tank_packet.unk2;
                        }
                    }
                    ETankPacketType::NetGamePacketSendTileUpdateData => {
                        let tile = {
                            let world = bot.world.write().unwrap();
                            match world.get_tile(tank_packet.int_x as u32, tank_packet.int_y as u32) {
                                Some(tile) => {
                                    tile.clone()
                                }
                                None => {
                                    return;
                                }
                            }
                        };
                        let data = &data[56..];
                        let mut cursor = Cursor::new(data);
                        bot.world
                            .write()
                            .unwrap()
                            .update_tile(tile, &mut cursor, true);
                    }
                    ETankPacketType::NetGamePacketSendItemDatabaseData => {
                        let data = &data[56..];
                        let mut decoder = ZlibDecoder::new(data);
                        let mut data = Vec::new();
                        decoder.read_to_end(&mut data).unwrap();
                        fs::write("items.dat", &data).unwrap();
                        let mut item_database = bot.item_database.write().unwrap();
                        *item_database = gtitem_r::load_from_memory(&data).unwrap();
                    }
                    _ => {}
                }
            }
            Err(..) => {
                bot.log_error(format!("Failed to deserialize TankPacket: {:?}", data[0]).as_str());
            }
        },
        EPacketType::NetMessageClientLogRequest => {
            let message = String::from_utf8_lossy(&data);
            bot.log_info(format!("Message: {}", message).as_str());
        }
        EPacketType::NetMessageTrack => {
            let message = String::from_utf8_lossy(&data);
            let data = utils::textparse::parse_and_store_as_map(&message);
            if data.contains_key("Level") {
                let level = data.get("Level").unwrap();
                let mut state = bot.state.lock().unwrap();
                state.level = level.parse().unwrap();
            }
            if data.contains_key("Global_Playtime") {
                let playtime = data.get("Global_Playtime").unwrap();
                let mut state = bot.state.lock().unwrap();
                state.playtime = playtime.parse().unwrap();
            }
            if data.contains_key("installDate") {
                let install_date = data.get("installDate").unwrap();
                let mut state = bot.state.lock().unwrap();
                state.install_date = install_date.parse().unwrap();
            }
        }
        _ => (),
    }
}
