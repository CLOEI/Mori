/*
1.  `oLock the World``|Open inventory and place a `2My First World Lock``.|1|interface/tutorial/tut_npc.rttex|Open inventory and place a `2My First World Lock``.|1
2.  `oBreak Dirt Blocks``|Select the `2Fist`` and break some `2Dirt``!|2|interface/tutorial/tut_npc.rttex|Select the `2Fist`` and break some `2Dirt``!|1
3.  `oCollect Dirt Seeds``|Break the `2Dirt`` to collect `2Dirt Seeds``.|3|interface/tutorial/tut_npc.rttex|Break the `2Dirt`` to collect `2Dirt Seeds``.|1
4.  `oPlant Dirt Seeds``|Plant `2Dirt Seeds`` on the ground to grow a `2Dirt Tree``.|4|interface/tutorial/tut_npc.rttex|Plant `2Dirt Seeds`` on the ground to grow a `2Dirt Tree``.|1
5.  `oHarvest Dirt Trees``|Harvest the `2Dirt Tree`` that you planted!|5|interface/tutorial/tut_npc.rttex|Harvest the `2Dirt Tree`` that you planted!|1
6.
 */
use std::sync::Arc;
use std::thread;
use crate::bot;
use crate::bot::Bot;
use crate::types::epacket_type::EPacketType;

static DIRT: u16 = 2;
static DIRT_SEEDS: u16 = 3;

pub fn lock_the_world(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();
    let current_task_is_lock = {
        let ftue = bot.ftue.read().unwrap();
        if ftue.info.contains("`oLock the World`") {
            true
        } else {
            false
        }
    };

    if !current_task_is_lock {
        return;
    }

    thread::spawn(move || {
        bot::send_packet(&bot_clone, EPacketType::NetMessageGenericText, "ftue_start_popup_close`".to_string());
        thread::sleep(std::time::Duration::from_millis(1000));
        bot::place(&bot_clone, 0, -1, 9640);
        thread::sleep(std::time::Duration::from_millis(250));
    });
}

pub fn break_dirt_block(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();

    thread::spawn(move || {
        let tiles = {
            let world = bot_clone.world.read().unwrap();
            world.tiles.clone()
        };

        loop {
            let current_task_is_break_dirt = {
                let ftue = bot_clone.ftue.read().unwrap();
                if ftue.info.contains("`oBreak Dirt Blocks`") {
                    true
                } else {
                    false
                }
            };

            if !current_task_is_break_dirt {
                break;
            }

            for tile in tiles.iter() {
                if tile.foreground_item_id == DIRT {
                    loop {
                        let (foreground_id) = {
                            let world = bot_clone.world.read().unwrap();
                            if let Some(tile) = world.get_tile(tile.x, tile.y) {
                                (tile.foreground_item_id)
                            } else {
                                break;
                            }
                        };

                        if foreground_id != DIRT {
                            break;
                        }

                        bot::find_path(&bot_clone, tile.x, tile.y - 1);
                        thread::sleep(std::time::Duration::from_millis(100));
                        bot::punch(&bot_clone, 0, 1);
                        thread::sleep(std::time::Duration::from_millis(250));
                    }
                }
            }
        }
    });
}

pub fn plant_dirt_seed(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();

    thread::spawn(move || {
        let tiles = {
            let world = bot_clone.world.read().unwrap();
            world.tiles.clone()
        };

        loop {
            let current_task_is_plant_dirt_seeds = {
                let ftue = bot_clone.ftue.read().unwrap();
                if ftue.info.contains("`oPlant Dirt Seeds`") {
                    true
                } else {
                    false
                }
            };

            if !current_task_is_plant_dirt_seeds {
                break;
            }

            for tile in tiles.iter() {
                if tile.foreground_item_id == DIRT {
                    let (foreground_id) = {
                        let world = bot_clone.world.read().unwrap();
                        if let Some(tile) = world.get_tile(tile.x, tile.y - 1) {
                            (tile.foreground_item_id)
                        } else {
                            break;
                        }
                    };

                    if foreground_id == 0 {
                        bot::find_path(&bot_clone, tile.x, tile.y - 1);
                        thread::sleep(std::time::Duration::from_millis(100));
                        bot::place(&bot_clone, 0, 0, DIRT_SEEDS as u32);
                        thread::sleep(std::time::Duration::from_millis(250));
                    }
                }
            }
        }
    });
}