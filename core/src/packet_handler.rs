use crate::types::net_game_packet::{NetGamePacket, NetGamePacketData};
use crate::types::net_message::NetMessage;
use crate::utils::proton::HashMode;
use crate::{Bot, utils, variant_handler};
use byteorder::{ByteOrder, LittleEndian};
use flate2::read::ZlibDecoder;
use std::fs;
use std::io::Read;
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
                NetGamePacket::CallFunction => {
                    variant_handler::handle(bot, &data[60..]);
                }
                NetGamePacket::SendMapData => {
                    let mut is_inworld_lock = bot.is_inworld.lock().unwrap();
                    *is_inworld_lock = true;

                    let world_data = &data[60..];
                    fs::write("world.dat", world_data).expect("Unable to write world data");
                    let item_database_lock = bot.item_database.read().unwrap();
                    let item_database = item_database_lock.deref();
                    let mut world_lock = bot.world.data.lock().unwrap();
                    let _ =world_lock.parse(&data[60..], item_database);
                }
                NetGamePacket::SendInventoryState => {
                    bot.inventory.lock().unwrap().parse(&data[60..])
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

                    let (in_world, net_id) = {
                        let state = bot.is_inworld.lock().unwrap();
                        let net_id = bot.net_id.lock().unwrap();
                        (*state, *net_id)
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

                    let item_database =
                        gtitem_r::load_from_file("items.dat").expect("Failed to load items.dat");
                    // bot.item_database = Arc::new(item_database);
                }
                _ => {}
            }
        }
        _ => {}
    }
}
