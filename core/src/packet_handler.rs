use crate::types::net_game_packet::{NetGamePacket, NetGamePacketData};
use crate::types::net_message::NetMessage;
use crate::utils::proton::HashMode;
use crate::{Bot, utils, variant_handler};
use byteorder::{ByteOrder, LittleEndian};
use flate2::read::ZlibDecoder;
use std::fs;
use std::io::{Cursor, Read};
use std::ops::Deref;

pub fn handle(bot: &Bot, data: &[u8]) {
    let packet_id = LittleEndian::read_u32(&data[0..4]);
    let packet_type = NetMessage::from(packet_id);

    match packet_type {
        NetMessage::ServerHello => {
            let is_redirecting = {
                let is_redirecting_lock = bot.is_redirecting.lock().unwrap();
                *is_redirecting_lock
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
                    let mut is_redirecting_lock = bot.is_redirecting.lock().unwrap();
                    *is_redirecting_lock = false;
                }
            } else {
                data = format!(
                    "protocol|{}\nltoken|{}\nplatformID|{}\n",
                    login_info.protocol, login_info.ltoken, login_info.platform_id
                );
            }
            bot.send_packet(NetMessage::GenericText, data.as_bytes(), None, true);
        }
        NetMessage::GameMessage => {
            let message = String::from_utf8_lossy(&data[4..]).to_string();
            println!("GameMessage: {}", message);

            if message.contains("logon_fail") {
                bot.disconnect()
            }
        }
        NetMessage::GamePacket => {
            let parsed = NetGamePacketData::from_bytes(&data[4..])
                .expect("Failed to parse NetGamePacketData");
            println!("GamePacket: {:?}", parsed._type);
            match parsed._type {
                NetGamePacket::State => {
                    let mut players = bot.world.players.lock().unwrap();
                    if let Some(player) = players.get_mut(&parsed.net_id) {
                        player.position.0 = parsed.vector_x;
                        player.position.1 = parsed.vector_y;
                    }
                }
                NetGamePacket::CallFunction => {
                    variant_handler::handle(bot, &data[60..]);
                }
                NetGamePacket::SendMapData => {
                    let world_data = &data[60..];
                    fs::write("world.dat", world_data).expect("Unable to write world data");
                    let item_database_lock = bot.item_database.read().unwrap();
                    let item_database = item_database_lock.deref();
                    let mut world_lock = bot.world.data.lock().unwrap();
                    let _ = world_lock.parse(&data[60..], &item_database);
                    
                    if !world_lock.tiles.is_empty() {
                        let mut collision_data = Vec::with_capacity(world_lock.tiles.len());
                        for tile in &world_lock.tiles {
                            let collision_type = if let Some(item) = item_database.get_item(&(tile.foreground_item_id as u32)) {
                                item.collision_type
                            } else {
                                0
                            };
                            collision_data.push(collision_type);
                        }
                        
                        let mut astar_lock = bot.astar.lock().unwrap();
                        astar_lock.update_from_collision_data(world_lock.width, world_lock.height, &collision_data);
                    }
                }
                NetGamePacket::SendInventoryState => {
                    bot.inventory.parse(&data[60..])
                }
                NetGamePacket::SetCharacterState => {
                    let hack_type = parsed.value;
                    let build_length = parsed.jump_count - 126;
                    let punch_length = parsed.animation_type - 126;
                    let gravity = parsed.vector_x2;
                    let velocity = parsed.vector_y2;

                    let mut state_lock = bot.state.lock().unwrap();
                    state_lock.hack_type = hack_type;
                    state_lock.build_length = build_length;
                    state_lock.punch_length = punch_length;
                    state_lock.velocity = velocity;
                    state_lock.gravity = gravity;
                }
                NetGamePacket::PingRequest => {
                    let elapsed = {
                        let duration_lock = bot.duration.lock().unwrap();
                        let duration = duration_lock.elapsed();
                        duration.as_millis() as u32
                    };

                    let value = parsed.value;
                    let (hack_type, build_length, punch_length, gravity, velocity) = {
                        let state_lock = bot.state.lock().unwrap();
                        (
                            state_lock.hack_type,
                            state_lock.build_length,
                            state_lock.punch_length,
                            state_lock.gravity,
                            state_lock.velocity,
                        )
                    };

                    let mut data = NetGamePacketData {
                        _type: NetGamePacket::PingReply,
                        target_net_id: utils::proton::hash(
                            value.to_string().as_bytes(),
                            HashMode::NullTerminated,
                        ) as i32,
                        value: elapsed,
                        vector_x: (if build_length == 0 {
                            2.0
                        } else {
                            build_length as f32
                        }) * 32.0,
                        vector_y: (if punch_length == 0 {
                            2.0
                        } else {
                            punch_length as f32
                        }) * 32.0,
                        ..Default::default()
                    };

                    let in_world = {
                        let world_lock = bot.world.data.lock().unwrap();
                        let in_world = world_lock.name != "EXIT";
                        in_world
                    };

                    if in_world {
                        data.net_id = hack_type;
                        data.vector_x2 = velocity;
                        data.vector_y2 = gravity;
                    }

                    bot.send_packet(NetMessage::GamePacket, &data.to_bytes(), None, true);
                }
                NetGamePacket::SendItemDatabaseData => {
                    let data = &data[60..];
                    let mut decoder = ZlibDecoder::new(data);
                    let mut data = Vec::new();
                    decoder.read_to_end(&mut data).unwrap();
                    fs::write("items.dat", &data).unwrap();

                    bot.send_packet(
                        NetMessage::GenericText,
                        "action|enter_game\n".to_string().as_bytes(),
                        None,
                        true,
                    );
                    let mut is_redirecting_lock = bot.is_redirecting.lock().unwrap();
                    *is_redirecting_lock = false;

                    let item_database = gtitem_r::load_from_file("items.dat").expect("Failed to load items.dat");
                    *bot.item_database.write().unwrap() = item_database;
                }
                NetGamePacket::TileChangeRequest => {
                    handle_tile_change_request(bot, &parsed);
                }
                NetGamePacket::ItemChangeObject => {
                    handle_item_change_object(bot, &parsed);
                }
                NetGamePacket::SendTileTreeState => {
                    handle_send_tile_tree_state(bot, &parsed);
                }
                NetGamePacket::ModifyItemInventory => {
                    handle_modify_item_inventory(bot, &parsed);
                }
                NetGamePacket::SendTileUpdateData => {
                    handle_send_tile_update_data(bot, &parsed, &data);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn handle_tile_change_request(bot: &Bot, tank_packet: &NetGamePacketData) {
    if tank_packet.value == 18 {
        update_tile_for_punch(bot, tank_packet);
        update_single_tile_astar(bot, tank_packet.int_x as u32, tank_packet.int_y as u32, 0);
        return;
    }

    let should_update_inventory = {
        let net_id = bot.net_id.lock().unwrap();
        *net_id == tank_packet.net_id
    };

    if should_update_inventory {
        update_inventory_for_tile_change(bot, tank_packet);
    }

    update_tile_for_place(bot, tank_packet);
    
    let collision_type = get_collision_type_for_item(bot, tank_packet.value);
    update_single_tile_astar(bot, tank_packet.int_x as u32, tank_packet.int_y as u32, collision_type);
}

fn handle_item_change_object(bot: &Bot, tank_packet: &NetGamePacketData) {
    let mut world = bot.world.data.lock().unwrap();
    
    match tank_packet.net_id {
        u32::MAX => {
            let item = gtworld_r::DroppedItem {
                id: tank_packet.value as u16,
                x: tank_packet.vector_x.ceil(),
                y: tank_packet.vector_y.ceil(),
                count: tank_packet.float_variable as u8,
                flags: tank_packet.object_type,
                uid: world.dropped.last_dropped_item_uid + 1,
            };

            world.dropped.items.push(item);
            world.dropped.last_dropped_item_uid += 1;
            world.dropped.items_count += 1;
        }
        net_id if net_id == u32::MAX - 3 => {
            if let Some(obj) = world.dropped.items.iter_mut().find(|obj| {
                obj.id == tank_packet.value as u16
                    && obj.x == tank_packet.vector_x.ceil()
                    && obj.y == tank_packet.vector_y.ceil()
            }) {
                obj.count = tank_packet.jump_count;
            }
        }
        net_id if net_id > 0 => {
            if let Some((index, collected_item)) = world
                .dropped
                .items
                .iter()
                .enumerate()
                .find(|(_, obj)| obj.uid == tank_packet.value)
                .map(|(i, obj)| (i, obj.clone()))
            {
                let our_net_id = *bot.net_id.lock().unwrap();
                if tank_packet.net_id == our_net_id {
                    update_player_inventory_from_dropped_item(bot, &collected_item);
                }

                world.dropped.items.remove(index);
                world.dropped.items_count -= 1;
            }
        }
        _ => {}
    }
    
    drop(world);
}


fn handle_send_tile_tree_state(bot: &Bot, tank_packet: &NetGamePacketData) {
    let mut world = bot.world.data.lock().unwrap();
    if let Some(tile) = world.get_tile_mut(tank_packet.int_x as u32, tank_packet.int_y as u32) {
        tile.foreground_item_id = 0;
        tile.tile_type = gtworld_r::TileType::Basic;
    }
    drop(world);
    
    update_single_tile_astar(bot, tank_packet.int_x as u32, tank_packet.int_y as u32, 0);
}

fn update_inventory_for_tile_change(bot: &Bot, tank_packet: &NetGamePacketData) {
    let item_id = tank_packet.value as u16;
    bot.inventory.remove_item(item_id, 1);
}

fn update_tile_for_punch(bot: &Bot, tank_packet: &NetGamePacketData) {
    let mut world = bot.world.data.lock().unwrap();
    if let Some(tile) = world.get_tile_mut(tank_packet.int_x as u32, tank_packet.int_y as u32) {
        if tile.foreground_item_id != 0 {
            tile.foreground_item_id = 0;
        } else {
            tile.background_item_id = 0;
        }
    }
}

fn update_tile_for_place(bot: &Bot, tank_packet: &NetGamePacketData) {
    let mut world = bot.world.data.lock().unwrap();
    if let Some(tile) = world.get_tile_mut(tank_packet.int_x as u32, tank_packet.int_y as u32) {
        let item_database = bot.item_database.read().unwrap();
        if let Some(item) = item_database.items.get(&tank_packet.value) {
            if matches!(item.action_type, 22 | 28 | 18) {
                tile.background_item_id = tank_packet.value as u16;
            } else {
                tile.foreground_item_id = tank_packet.value as u16;
                
                if item.id % 2 != 0 {
                    tile.tile_type = gtworld_r::TileType::Seed {
                        ready_to_harvest: false,
                        time_passed: 0,
                        item_on_tree: 0,
                        elapsed: std::time::Instant::now().elapsed(),
                    };
                }
            }
        }
    }
}

fn update_player_inventory_from_dropped_item(bot: &Bot, dropped_item: &gtworld_r::DroppedItem) {
    if dropped_item.id == 112 {
        bot.inventory.add_gems(dropped_item.count as i32);
    } else {
        bot.inventory.add_item(dropped_item.id, dropped_item.count);
    }
}

fn get_collision_type_for_item(bot: &Bot, item_id: u32) -> u8 {
    let item_database = bot.item_database.read().unwrap();
    if let Some(item) = item_database.get_item(&item_id) {
        item.collision_type
    } else {
        0
    }
}

fn update_single_tile_astar(bot: &Bot, x: u32, y: u32, new_collision_type: u8) {
    let mut astar = bot.astar.lock().unwrap();
    astar.update_single_tile(x, y, new_collision_type);
}

fn handle_modify_item_inventory(bot: &Bot, tank_packet: &NetGamePacketData) {
    let item_id = tank_packet.value as u16;
    let amount_to_remove = tank_packet.jump_count;

    bot.inventory.remove_item(item_id, amount_to_remove);
}

fn handle_send_tile_update_data(bot: &Bot, tank_packet: &NetGamePacketData, data: &[u8]) {
    let tile_x = tank_packet.int_x as u32;
    let tile_y = tank_packet.int_y as u32;
    
    let world_bounds = {
        let world = bot.world.data.lock().unwrap();
        (world.width, world.height)
    };
    
    if tile_x >= world_bounds.0 || tile_y >= world_bounds.1 {
        return;
    }
    
    let old_collision_type = {
        let world = bot.world.data.lock().unwrap();
        if let Some(tile) = world.get_tile(tile_x, tile_y) {
            let item_database = bot.item_database.read().unwrap();
            if let Some(item) = item_database.get_item(&(tile.foreground_item_id as u32)) {
                item.collision_type
            } else {
                0
            }
        } else {
            return;
        }
    };
    
    let tile_data = &data[56..];
    let mut cursor = Cursor::new(tile_data);
    
    let new_collision_type = {
        let item_database = bot.item_database.read().unwrap();
        let mut world = bot.world.data.lock().unwrap();
        if let Some(tile) = world.get_tile(tile_x, tile_y) {
            let tile_clone = tile.clone();
            let _ = world.update_tile(tile_clone, &mut cursor, true, &item_database);
            
            if let Some(updated_tile) = world.get_tile(tile_x, tile_y) {
                if let Some(item) = item_database.get_item(&(updated_tile.foreground_item_id as u32)) {
                    item.collision_type
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        }
    };
    
    if old_collision_type != new_collision_type {
        update_single_tile_astar(bot, tile_x, tile_y, new_collision_type);
    }
}
