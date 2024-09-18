/*
1.  `oLock the World``|Open inventory and place a `2My First World Lock``.|1|interface/tutorial/tut_npc.rttex|Open inventory and place a `2My First World Lock``.|1
2.  `oBreak Dirt Blocks``|Select the `2Fist`` and break some `2Dirt``!|2|interface/tutorial/tut_npc.rttex|Select the `2Fist`` and break some `2Dirt``!|1
3.  `oCollect Dirt Seeds``|Break the `2Dirt`` to collect `2Dirt Seeds``.|3|interface/tutorial/tut_npc.rttex|Break the `2Dirt`` to collect `2Dirt Seeds``.|1
4.  `oPlant Dirt Seeds``|Plant `2Dirt Seeds`` on the ground to grow a `2Dirt Tree``.|4|interface/tutorial/tut_npc.rttex|Plant `2Dirt Seeds`` on the ground to grow a `2Dirt Tree``.|1
5.  `oHarvest Dirt Trees``|Harvest the `2Dirt Tree`` that you planted!|5|interface/tutorial/tut_npc.rttex|Harvest the `2Dirt Tree`` that you planted!|1
6.  `oBreak Rock Blocks``|Select the `2Fist`` and break some `2Rock``!|19|interface/tutorial/tut_npc.rttex|Select the `2Fist`` and break some `2Rock``!|1
7.  `oCollect Rock Seeds``|Break the `2Rock`` to collect `2Rock Seeds``.|6|interface/tutorial/tut_npc.rttex|Break the `2Rock`` to collect `2Rock Seeds``.|1
8.  `oBreak Cave Backgrounds``|Select the `2Fist`` and break some `2Cave Background``!|20|interface/tutorial/tut_npc.rttex|Select the `2Fist`` and break some `2Cave Background``!|1
9.  `oCollect Cave Background Seeds``|Break the `2Cave Background`` to collect `2Cave Background Seeds``.|14|interface/tutorial/tut_npc.rttex|Break the `2Cave Background`` to collect `2Cave Background Seeds``.|1
10. `oSplice Rock and Cave Background Seeds``|Splice `2Rock`` and `2Cave Background`` Seeds by planting them both on the same tile.|15|interface/tutorial/tut_npc.rttex|Splice `2Rock`` and `2Cave Background`` Seeds by planting them both on the same tile.|1
11. `oPlace a Sign in the World``|Collect the `2Sign`` block that you have grown in the world.|16|interface/tutorial/tut_npc.rttex|Collect the `2Sign`` block that you have grown in the world.|1
12. `oWrench the Sign that you placed``|Wrench the `2Sign`` to change what it says!|17|interfae/tutorial/tut_npc.rttex|Wrench the `2Sign`` to change what it says!|1c
13. `oBreak Lava Blocks``|Select the `2Fist`` and break some `2Lava``!|21|interface/tutorial/tut_npc.rttex|Select the `2Fist`` and break some `2Lava``!|1
14.
15.
 */
use std::fmt::format;
use std::sync::Arc;
use std::thread;
use gtworld_r::TileType;
use crate::bot;
use crate::bot::Bot;
use crate::types::epacket_type::EPacketType;

static DIRT: u16 = 2;
static ROCK: u16 = 10;
static CAVE_BACKGROUND: u16 = 14;
static SIGN: u16 = 20;
static LAVA: u16 = 4;
static DIRT_SEEDS: u16 = 3;
static ROCK_SEED: u16 = 11;
static CAVE_BACKGROUND_SEED: u16 = 15;
static SIGN_SEED: u16 = 21;
static LAVA_SEED: u16 = 5;

pub fn lock_the_world(bot: &Arc<Bot>) {
    if !is_current_task(bot, "`oLock the World`") {
        return;
    }

    let bot_clone = bot.clone();
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
        while is_current_task(&bot_clone, "`oBreak Dirt Blocks`") {
            let tiles = {
                let world = bot_clone.world.read().unwrap();
                world.tiles.clone()
            };

            for tile in tiles.iter() {
                if tile.foreground_item_id == DIRT {
                    if !is_current_task(&bot_clone, "`oBreak Dirt Blocks`") {
                        return;
                    }

                    while {
                        let world = bot_clone.world.read().unwrap();
                        world.get_tile(tile.x, tile.y).unwrap().foreground_item_id == DIRT
                    } {
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
        while is_current_task(&bot_clone, "`oPlant Dirt Seeds`") {
            let tiles = {
                let world = bot_clone.world.read().unwrap();
                world.tiles.clone()
            };

            for tile in tiles.iter() {
                if tile.foreground_item_id == DIRT {
                    if !is_current_task(&bot_clone, "`oPlant Dirt Seeds`") {
                        return;
                    }

                    while {
                        let world = bot_clone.world.read().unwrap();
                        world.get_tile(tile.x, tile.y - 1).unwrap().foreground_item_id == 0
                    } {
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

pub fn harvest_dirt_tree(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();

    thread::spawn(move || {
        while is_current_task(&bot_clone, "`oHarvest Dirt Trees`") {
            let tiles = {
                let world = bot_clone.world.read().unwrap();
                world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == DIRT_SEEDS).collect::<Vec<_>>()
            };

            for tile in tiles.iter() {
                if !is_current_task(&bot_clone, "`oHarvest Dirt Trees`") {
                    return;
                }

                while {
                    let world = bot_clone.world.read().unwrap();
                    world.is_tile_harvestable(&tile)
                } {
                    bot::find_path(&bot_clone, tile.x, tile.y);
                    thread::sleep(std::time::Duration::from_millis(100));
                    bot::punch(&bot_clone, 0, 0);
                    thread::sleep(std::time::Duration::from_millis(250));
                }
            }
        }
    });
}

pub fn break_rock_block(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();

    thread::spawn(move || {
        while is_current_task(&bot_clone, "`oBreak Rock Blocks`") {
            let rock_tree_tiles = {
                let world = bot_clone.world.read().unwrap();
                world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == ROCK_SEED).collect::<Vec<_>>()
            };

            for tile in rock_tree_tiles.iter() {
                let rock_amount = {
                    let inventory = bot_clone.inventory.read().unwrap();
                    inventory.items.get(&ROCK).map_or(0, |item| item.amount)
                };

                if rock_amount >= 5 {
                    return;
                }

                bot::find_path(&bot_clone, tile.x, tile.y);
                thread::sleep(std::time::Duration::from_millis(100));
                bot::punch(&bot_clone, 0, 0);
                thread::sleep(std::time::Duration::from_millis(250));
            }

            bot::find_path(&bot_clone, 0, 0);
            thread::sleep(std::time::Duration::from_millis(100));
            while is_current_task(&bot_clone, "`oBreak Rock Blocks`") {
                bot::place(&bot_clone, 1, 0, ROCK as u32);
                thread::sleep(std::time::Duration::from_millis(100));

                while {
                    let world = &bot_clone.world.read().unwrap();
                    world.get_tile(1, 0).map_or(false, |tile| tile.foreground_item_id == ROCK)
                } {
                    bot::punch(&bot_clone, 1, 0);
                    thread::sleep(std::time::Duration::from_millis(250));
                }
            }
        }
    });
}

pub fn collect_rock_seed(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();

    thread::spawn(move || {
        while is_current_task(&bot_clone, "`oCollect Rock Seeds`") {
            let rock_tree_tiles = {
                let world = bot_clone.world.read().unwrap();
                world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == ROCK_SEED).collect::<Vec<_>>()
            };

            for tile in rock_tree_tiles.iter() {
                if !is_current_task(&bot_clone, "`oCollect Rock Seeds`") {
                    return;
                }

                bot::find_path(&bot_clone, tile.x, tile.y);
                thread::sleep(std::time::Duration::from_millis(100));
                bot::punch(&bot_clone, 0, 0);
                thread::sleep(std::time::Duration::from_millis(250));
            }

            bot::find_path(&bot_clone, 0, 0);
            thread::sleep(std::time::Duration::from_millis(100));

            while is_current_task(&bot_clone, "`oCollect Rock Seeds`") {
                bot::place(&bot_clone, 1, 0, ROCK as u32);
                thread::sleep(std::time::Duration::from_millis(100));

                while {
                    let world = bot_clone.world.read().unwrap();
                    world.get_tile(1, 0).map_or(false, |tile| tile.foreground_item_id == ROCK)
                } {
                    bot::punch(&bot_clone, 1, 0);
                    thread::sleep(std::time::Duration::from_millis(250));
                }
            }
        }
    });
}

pub fn break_cave_background(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();

    thread::spawn(move || {
        while is_current_task(&bot_clone, "`oBreak Cave Backgrounds`") {
            let dirt_tiles = {
                let world = bot_clone.world.read().unwrap();
                world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == DIRT).collect::<Vec<_>>()
            };

            for tile in dirt_tiles.iter() {
                if !is_current_task(&bot_clone, "`oBreak Cave Backgrounds`") {
                    return;
                }

                while {
                    let world = bot_clone.world.read().unwrap();
                    let tile = world.get_tile(tile.x, tile.y).unwrap();
                    tile.background_item_id != 0 || tile.foreground_item_id != 0
                } {
                    bot::find_path(&bot_clone, tile.x, tile.y - 1);
                    thread::sleep(std::time::Duration::from_millis(100));
                    bot::punch(&bot_clone, 0, 1);
                    thread::sleep(std::time::Duration::from_millis(250));
                }
            }
        }
    });
}

pub fn collect_cave_background_seed(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();

    thread::spawn(move || {
        while is_current_task(&bot_clone, "`oCollect Cave Background Seeds`") {
            let cbg_tree_tiles = {
                let world = bot_clone.world.read().unwrap();
                world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == CAVE_BACKGROUND_SEED).collect::<Vec<_>>()
            };

            for tile in cbg_tree_tiles.iter() {
                if !is_current_task(&bot_clone, "`oCollect Cave Background Seeds`") {
                    return;
                }

                bot::find_path(&bot_clone, tile.x, tile.y);
                thread::sleep(std::time::Duration::from_millis(100));
                bot::punch(&bot_clone, 0, 0);
                thread::sleep(std::time::Duration::from_millis(250));
            }

            bot::find_path(&bot_clone, 0, 0);
            thread::sleep(std::time::Duration::from_millis(100));

            while is_current_task(&bot_clone, "`oCollect Cave Background Seeds`") {
                bot::place(&bot_clone, 1, 0, ROCK as u32);
                thread::sleep(std::time::Duration::from_millis(100));

                while {
                    let world = bot_clone.world.read().unwrap();
                    world.get_tile(1, 0).map_or(false, |tile| tile.foreground_item_id == CAVE_BACKGROUND)
                } {
                    bot::punch(&bot_clone, 1, 0);
                    thread::sleep(std::time::Duration::from_millis(250));
                }
            }
        }
    });
}

pub fn splice_rock_and_cbg_seed(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();

    thread::spawn(move || {
        bot::find_path(&bot_clone, 0, 0);
        thread::sleep(std::time::Duration::from_millis(100));
        bot::place(&bot_clone, 1, 1, ROCK as u32);
        thread::sleep(std::time::Duration::from_millis(100));
        bot::place(&bot_clone, 1, 0, CAVE_BACKGROUND_SEED as u32);
        thread::sleep(std::time::Duration::from_millis(100));
        bot::place(&bot_clone, 1, 0, ROCK_SEED as u32);
        thread::sleep(std::time::Duration::from_millis(100));
    });
}

pub fn place_sign_in_world(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();

    thread::spawn(move || {
        loop {
            let (sign_tree_tile, is_harvestable, sign_count) = {
                let world = bot_clone.world.read().unwrap();
                let inventory = bot_clone.inventory.read().unwrap();
                let sign_count = inventory.items.get(&SIGN).map_or(0, |item| item.amount);
                let tile = world.tiles.clone().into_iter().find(|tile| tile.foreground_item_id == SIGN_SEED).unwrap();
                let is_harvestable = world.is_tile_harvestable(&tile);
                (tile, is_harvestable, sign_count)
            };

            if is_harvestable {
                bot::find_path(&bot_clone, sign_tree_tile.x, sign_tree_tile.y);
                thread::sleep(std::time::Duration::from_millis(100));
                bot::punch(&bot_clone, 0, 0);
                thread::sleep(std::time::Duration::from_millis(250));
                break;
            }

            if sign_count > 0 {
                bot::place(&bot_clone, 0, 0, SIGN as u32);
                thread::sleep(std::time::Duration::from_millis(250));
            }
        }
    });
}

pub fn wrench_sign(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();

    thread::spawn(move || {
        let sign_tile = {
            let world = bot_clone.world.read().unwrap();
            let tile = world.tiles.clone().into_iter().find(|tile| tile.foreground_item_id == SIGN);
            (tile)
        };

        if sign_tile.is_some() {
            let sign_tile = sign_tile.unwrap();
            bot::find_path(&bot_clone, sign_tile.x, sign_tile.y);
            thread::sleep(std::time::Duration::from_millis(100));
            bot::wrench(&bot_clone, 0, 0);
            thread::sleep(std::time::Duration::from_millis(1000));
            bot::send_packet(&bot_clone, EPacketType::NetMessageGenericText, format!("action|dialog_return\ndialog_name|sign_edit\ntilex|{}|\ntiley|{}|\nsign_text|CLOEI\n", sign_tile.x, sign_tile.y).to_string());
            thread::sleep(std::time::Duration::from_millis(1000));
            while {
                let world = bot_clone.world.read().unwrap();
                world.get_tile(1, 0).map_or(false, |tile| tile.foreground_item_id == SIGN)
            } {
                bot::punch(&bot_clone, 0, 0);
                thread::sleep(std::time::Duration::from_millis(250));
            }
        }
    });
}

pub fn harvest_and_break_lava(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();

    thread::spawn(move || {
        let lava_tree_tiles = {
            let world = bot_clone.world.read().unwrap();
            let tiles = world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == LAVA_SEED).collect::<Vec<_>>();
            (tiles)
        };

        for tile in lava_tree_tiles.iter() {
            let (is_harvestable, lava_count) = {
                let world = bot_clone.world.read().unwrap();
                let inventory = bot_clone.inventory.read().unwrap();
                let lava_count = inventory.items.get(&LAVA).map_or(0, |item| item.amount);
                (world.is_tile_harvestable(&tile), lava_count)
            };

            if lava_count > 5 {
                break;
            }

            if is_harvestable && lava_count < 5 {
                bot::find_path(&bot_clone, tile.x, tile.y);
                thread::sleep(std::time::Duration::from_millis(100));
                bot::punch(&bot_clone, 0, 0);
                thread::sleep(std::time::Duration::from_millis(250));
            }
        }

        bot::find_path(&bot_clone, 0, 0);
        thread::sleep(std::time::Duration::from_millis(100));
        while is_current_task(&bot_clone, "`oBreak Lava Blocks`") {
            bot::place(&bot_clone, 1, 0, LAVA as u32);
            thread::sleep(std::time::Duration::from_millis(250));

            while {
                let world = bot_clone.world.read().unwrap();
                world.get_tile(1, 0).map_or(false, |tile| tile.foreground_item_id == LAVA)
            } {
                bot::punch(&bot_clone, 1, 0);
                thread::sleep(std::time::Duration::from_millis(250));
            }
        }
    });
}

fn is_current_task(bot: &Arc<Bot>, task: &str) -> bool {
    let ftue = bot.ftue.read().unwrap();
    ftue.info.contains(task)
}