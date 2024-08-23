use std::{fs, sync::Arc};

use paris::{error, info, warn};

use crate::{
    bot::{disconnect, send_packet, variant_handler},
    types::{
        epacket_type::EPacketType, etank_packet_type::ETankPacketType, tank_packet::TankPacket,
    },
};

use super::{inventory::InventoryItem, send_packet_raw, Bot};

pub fn handle(bot: &Arc<Bot>, packet_type: EPacketType, data: &[u8]) {
    match packet_type {
        EPacketType::NetMessageServerHello => {
            if bot.state.lock().unwrap().is_redirecting {
                let info = bot.info.lock().unwrap();
                let message = format!(
                    "UUIDToken|{}\nprotocol|{}\nfhash|{}\nmac|{}\nrequestedName|{}\nhash2|{}\nfz|{}\nf|{}\nplayer_age|{}\ngame_version|{}\nlmode|{}\ncbits|{}\nrid|{}\nGDPR|{}\nhash|{}\ncategory|{}\ntoken|{}\ntotal_playtime|{}\ndoor_id|{}\nklv|{}\nmeta|{}\nplatformID|{}\ndeviceVersion|{}\nzf|{}\ncountry|{}\nuser|{}\nwk|{}\n",
                    info.login_info.uuid, info.login_info.protocol, info.login_info.fhash, info.login_info.mac, info.login_info.requested_name, info.login_info.hash2, info.login_info.fz, info.login_info.f, info.login_info.player_age, info.login_info.game_version, info.login_info.lmode, info.login_info.cbits, info.login_info.rid, info.login_info.gdpr, info.login_info.hash, info.login_info.category, info.login_info.token, info.login_info.total_playtime, info.login_info.door_id, info.login_info.klv, info.login_info.meta, info.login_info.platform_id, info.login_info.device_version, info.login_info.zf, info.login_info.country, info.login_info.user, info.login_info.wk);
                send_packet(&bot, EPacketType::NetMessageGenericText, message);
            } else {
                let message = format!(
                    "protocol|{}\nltoken|{}\nplatformID|{}\n",
                    209,
                    bot.info.lock().unwrap().token,
                    "0,1,1"
                );
                send_packet(&bot, EPacketType::NetMessageGenericText, message);
            }
        }
        EPacketType::NetMessageGenericText => {}
        EPacketType::NetMessageGameMessage => {
            let message = String::from_utf8_lossy(&data);
            info!("Message: {}", message);

            if message.contains("logon_fail") {
                bot.state.lock().unwrap().is_redirecting = false;
                disconnect(bot);
            }
            if message.contains("currently banned") {
                {
                    let mut state = bot.state.lock().unwrap();
                    state.is_running = false;
                    state.is_banned = true;
                }
                disconnect(bot);
            }
            if message.contains("Advanced Account Protection") {
                bot.state.lock().unwrap().is_running = false;
                disconnect(bot);
            }
            if message.contains("temporarily suspended") {
                bot.state.lock().unwrap().is_running = false;
                disconnect(bot);
            }
        }
        EPacketType::NetMessageGamePacket => match bincode::deserialize::<TankPacket>(&data) {
            Ok(tank_packet) => match tank_packet._type {
                ETankPacketType::NetGamePacketState => {
                    for player in bot.players.lock().unwrap().iter_mut() {
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
                        value: tank_packet.value + 5000,
                        ..Default::default()
                    };

                    send_packet_raw(&bot, &packet);
                }
                ETankPacketType::NetGamePacketSendInventoryState => {
                    bot.inventory.lock().unwrap().parse(&data[56..]);
                }
                ETankPacketType::NetGamePacketSendMapData => {
                    warn!("Writing world.dat");
                    fs::write("world.dat", &data[56..]).unwrap();
                    // bot.world.lock().unwrap().parse(&data[56..]);
                    // bot.astar.lock().unwrap().update(bot);
                }
                ETankPacketType::NetGamePacketTileChangeRequest => {
                    if tank_packet.net_id == bot.state.lock().unwrap().net_id
                        && tank_packet.value != 18
                    {
                        for i in 0..bot.inventory.lock().unwrap().items.len() {
                            let mut inventory = bot.inventory.lock().unwrap();
                            if inventory.items[i].id == tank_packet.value as u16 {
                                inventory.items[i].amount -= 1;
                                if inventory.items[i].amount > 200 {
                                    inventory.items.remove(i);
                                }
                                break;
                            }
                        }
                    }

                    if let Some(tile) = bot
                        .world
                        .lock()
                        .unwrap()
                        .get_tile(tank_packet.int_x as u32, tank_packet.int_y as u32)
                    {
                        if tank_packet.value == 18 {
                            if tile.foreground_item_id != 0 {
                                tile.foreground_item_id = 0;
                            } else {
                                tile.background_item_id = 0;
                            }
                        } else {
                            if let Some(item) = bot.item_database.items.get(&tank_packet.value) {
                                if item.action_type == 22
                                    || item.action_type == 28
                                    || item.action_type == 18
                                {
                                    tile.background_item_id = tank_packet.value as u16;
                                } else {
                                    tile.foreground_item_id = tank_packet.value as u16;
                                }
                            }
                        }
                    }
                }
                ETankPacketType::NetGamePacketItemChangeObject => {
                    let mut world = bot.world.lock().unwrap();
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
                        for i in 0..world.dropped.items.len() {
                            let obj = &world.dropped.items[i];
                            if obj.uid == tank_packet.value {
                                if tank_packet.net_id == bot.state.lock().unwrap().net_id {
                                    if obj.id == 112 {
                                        bot.state.lock().unwrap().gems += obj.count as i32;
                                    } else {
                                        let mut added = false;
                                        for item in &mut bot.inventory.lock().unwrap().items {
                                            if item.id == obj.id {
                                                let temp = item.amount + obj.count as u16;
                                                item.amount = if temp > 200 { 200 } else { temp };
                                                added = true;
                                                break;
                                            }
                                        }
                                        if !added {
                                            let item = InventoryItem {
                                                id: obj.id,
                                                amount: obj.count as u16,
                                            };
                                            bot.inventory.lock().unwrap().items.push(item);
                                        }
                                    }
                                }
                                world.dropped.items.remove(i);
                                world.dropped.items_count -= 1;
                                break;
                            }
                        }
                    }
                }
                _ => {}
            },
            Err(..) => {
                error!("Failed to deserialize TankPacket: {:?}", data[0]);
            }
        },
        _ => (),
    }
}
