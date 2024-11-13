// /*
// 1.  `oLock the World``|Open inventory and place a `2My First World Lock``.|1|interface/tutorial/tut_npc.rttex|Open inventory and place a `2My First World Lock``.|1
// 2.  `oBreak Dirt Blocks``|Select the `2Fist`` and break some `2Dirt``!|2|interface/tutorial/tut_npc.rttex|Select the `2Fist`` and break some `2Dirt``!|1
// 3.  `oCollect Dirt Seeds``|Break the `2Dirt`` to collect `2Dirt Seeds``.|3|interface/tutorial/tut_npc.rttex|Break the `2Dirt`` to collect `2Dirt Seeds``.|1
// 4.  `oPlant Dirt Seeds``|Plant `2Dirt Seeds`` on the ground to grow a `2Dirt Tree``.|4|interface/tutorial/tut_npc.rttex|Plant `2Dirt Seeds`` on the ground to grow a `2Dirt Tree``.|1
// 5.  `oHarvest Dirt Trees``|Harvest the `2Dirt Tree`` that you planted!|5|interface/tutorial/tut_npc.rttex|Harvest the `2Dirt Tree`` that you planted!|1
// 6.  `oBreak Rock Blocks``|Select the `2Fist`` and break some `2Rock``!|19|interface/tutorial/tut_npc.rttex|Select the `2Fist`` and break some `2Rock``!|1
// 7.  `oCollect Rock Seeds``|Break the `2Rock`` to collect `2Rock Seeds``.|6|interface/tutorial/tut_npc.rttex|Break the `2Rock`` to collect `2Rock Seeds``.|1
// 8.  `oBreak Cave Backgrounds``|Select the `2Fist`` and break some `2Cave Background``!|20|interface/tutorial/tut_npc.rttex|Select the `2Fist`` and break some `2Cave Background``!|1
// 9.  `oCollect Cave Background Seeds``|Break the `2Cave Background`` to collect `2Cave Background Seeds``.|14|interface/tutorial/tut_npc.rttex|Break the `2Cave Background`` to collect `2Cave Background Seeds``.|1
// 10. `oSplice Rock and Cave Background Seeds``|Splice `2Rock`` and `2Cave Background`` Seeds by planting them both on the same tile.|15|interface/tutorial/tut_npc.rttex|Splice `2Rock`` and `2Cave Background`` Seeds by planting them both on the same tile.|1
// 11. `oPlace a Sign in the World``|Collect the `2Sign`` block that you have grown in the world.|16|interface/tutorial/tut_npc.rttex|Collect the `2Sign`` block that you have grown in the world.|1
// 12. `oWrench the Sign that you placed``|Wrench the `2Sign`` to change what it says!|17|interfae/tutorial/tut_npc.rttex|Wrench the `2Sign`` to change what it says!|1c
// 13. `oBreak Lava Blocks``|Select the `2Fist`` and break some `2Lava``!|21|interface/tutorial/tut_npc.rttex|Select the `2Fist`` and break some `2Lava``!|1
// 14. `oCollect Lava Seeds``|Break the `2Lava`` until Lava Seeds fall out!|7|interface/tutorial/tut_npc.rttex|Break the `2Lava`` until Lava Seeds fall out!|1
// 15. `oSplice Lava and Dirt Seeds``|Splice `2Lava Seeds`` and `2Dirt Seeds`` together by planting them both on the same tile.|8|interface/tutorial/tut_npc.rttex|Splice `2Lava Seeds`` and `2Dirt Seeds`` together by planting them both on the same tile.|1
// 16. `oBuild Wood Blocks``|Collect the `2Wood Blocks`` that you have grown in the world.|9|interface/tutorial/tut_npc.rttex|Collect the `2Wood Blocks`` that you have grown in the world.|1
// 17.
// 18.
//  */
// use std::sync::Arc;
// use std::thread;
// use crate::core;
// use crate::core::Bot;
// use crate::types::epacket_type::EPacketType;

// static DIRT: u16 = 2;
// static ROCK: u16 = 10;
// static CAVE_BACKGROUND: u16 = 14;
// static SIGN: u16 = 20;
// static LAVA: u16 = 4;
// static WOOD_BLOCK: u16 = 100;
// static DIRT_SEEDS: u16 = 3;
// static ROCK_SEED: u16 = 11;
// static CAVE_BACKGROUND_SEED: u16 = 15;
// static SIGN_SEED: u16 = 21;
// static LAVA_SEED: u16 = 5;
// static WOOD_BLOCK_SEED: u16 = 101;

// pub fn lock_the_world(bot: &Arc<Bot>) {
//     if !is_current_task(bot, "`oLock the World`") {
//         return;
//     }

//     bot.send_packet(EPacketType::NetMessageGenericText, "ftue_start_popup_close`".to_string());
//     thread::sleep(std::time::Duration::from_millis(1000));
//     bot.place(0, -1, 9640);
//     thread::sleep(std::time::Duration::from_millis(250));
// }

// // pub fn break_dirt_block(bot: &Arc<Bot>) {
// //     while is_current_task(&bot, "`oBreak Dirt Blocks`") {
// //         let tiles = {
// //             let world = &bot.world.read().unwrap();
// //             world.tiles.clone()
// //         };
// //
// //         for tile in tiles.iter() {
// //             if tile.foreground_item_id == DIRT {
// //                 if !is_current_task(&bot, "`oBreak Dirt Blocks`") {
// //                     return;
// //                 }
// //
// //                 while {
// //                     let world = &bot.world.read().unwrap();
// //                     world.get_tile(tile.x, tile.y).unwrap().foreground_item_id == DIRT
// //                 } {
// //                     core::find_path(&bot, tile.x, tile.y - 1);
// //                     thread::sleep(std::time::Duration::from_millis(100));
// //                     core::punch(&bot, 0, 1);
// //                     thread::sleep(std::time::Duration::from_millis(250));
// //                 }
// //             }
// //         }
// //     }
// // }
// //
// // pub fn plant_dirt_seed(bot: &Arc<Bot>) {
// //     while is_current_task(&bot, "`oPlant Dirt Seeds`") {
// //         let tiles = {
// //             let world = &bot.world.read().unwrap();
// //             world.tiles.clone()
// //         };
// //
// //         for tile in tiles.iter() {
// //             if tile.foreground_item_id == DIRT {
// //                 if !is_current_task(&bot, "`oPlant Dirt Seeds`") {
// //                     return;
// //                 }
// //
// //                 while {
// //                     let world = &bot.world.read().unwrap();
// //                     world.get_tile(tile.x, tile.y - 1).unwrap().foreground_item_id == 0
// //                 } {
// //                     core::find_path(&bot, tile.x, tile.y - 1);
// //                     thread::sleep(std::time::Duration::from_millis(100));
// //                     core::place(&bot, 0, 0, DIRT_SEEDS as u32);
// //                     thread::sleep(std::time::Duration::from_millis(250));
// //                 }
// //             }
// //         }
// //     }
// // }
// //
// // pub fn harvest_dirt_tree(bot: &Arc<Bot>) {
// //     while is_current_task(&bot, "`oHarvest Dirt Trees`") {
// //         let tiles = {
// //             let world = &bot.world.read().unwrap();
// //             world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == DIRT_SEEDS).collect::<Vec<_>>()
// //         };
// //
// //         for tile in tiles.iter() {
// //             if !is_current_task(&bot, "`oHarvest Dirt Trees`") {
// //                 return;
// //             }
// //
// //             while {
// //                 let world = &bot.world.read().unwrap();
// //                 world.is_tile_harvestable(&tile)
// //             } {
// //                 core::find_path(&bot, tile.x, tile.y);
// //                 thread::sleep(std::time::Duration::from_millis(100));
// //                 core::punch(&bot, 0, 0);
// //                 thread::sleep(std::time::Duration::from_millis(250));
// //             }
// //         }
// //     }
// // }
// //
// // pub fn break_rock_block(bot: &Arc<Bot>) {
// //     while is_current_task(&bot, "`oBreak Rock Blocks`") {
// //         let rock_tree_tiles = {
// //             let world = &bot.world.read().unwrap();
// //             world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == ROCK_SEED).collect::<Vec<_>>()
// //         };
// //
// //         for tile in rock_tree_tiles.iter() {
// //             let rock_amount = {
// //                 let inventory = &bot.inventory.read().unwrap();
// //                 inventory.items.get(&ROCK).map_or(0, |item| item.amount)
// //             };
// //
// //             if rock_amount >= 5 {
// //                 return;
// //             }
// //
// //             core::find_path(&bot, tile.x, tile.y);
// //             thread::sleep(std::time::Duration::from_millis(100));
// //             core::punch(&bot, 0, 0);
// //             thread::sleep(std::time::Duration::from_millis(250));
// //         }
// //
// //         core::find_path(&bot, 0, 0);
// //         thread::sleep(std::time::Duration::from_millis(100));
// //         while is_current_task(&bot, "`oBreak Rock Blocks`") {
// //             core::place(&bot, 1, 0, ROCK as u32);
// //             thread::sleep(std::time::Duration::from_millis(100));
// //
// //             while {
// //                 let world = &bot.world.read().unwrap();
// //                 world.get_tile(1, 0).map_or(false, |tile| tile.foreground_item_id == ROCK)
// //             } {
// //                 core::punch(&bot, 1, 0);
// //                 thread::sleep(std::time::Duration::from_millis(250));
// //             }
// //         }
// //     }
// // }
// //
// // pub fn collect_rock_seed(bot: &Arc<Bot>) {
// //     while is_current_task(&bot, "`oCollect Rock Seeds`") {
// //         let rock_tree_tiles = {
// //             let world = &bot.world.read().unwrap();
// //             world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == ROCK_SEED).collect::<Vec<_>>()
// //         };
// //
// //         for tile in rock_tree_tiles.iter() {
// //             if !is_current_task(&bot, "`oCollect Rock Seeds`") {
// //                 return;
// //             }
// //
// //             core::find_path(&bot, tile.x, tile.y);
// //             thread::sleep(std::time::Duration::from_millis(100));
// //             core::punch(&bot, 0, 0);
// //             thread::sleep(std::time::Duration::from_millis(250));
// //         }
// //
// //         core::find_path(&bot, 0, 0);
// //         thread::sleep(std::time::Duration::from_millis(100));
// //
// //         while is_current_task(&bot, "`oCollect Rock Seeds`") {
// //             core::place(&bot, 1, 0, ROCK as u32);
// //             thread::sleep(std::time::Duration::from_millis(100));
// //
// //             while {
// //                 let world = &bot.world.read().unwrap();
// //                 world.get_tile(1, 0).map_or(false, |tile| tile.foreground_item_id == ROCK)
// //             } {
// //                 core::punch(&bot, 1, 0);
// //                 thread::sleep(std::time::Duration::from_millis(250));
// //             }
// //         }
// //     }
// // }
// //
// // pub fn break_cave_background(bot: &Arc<Bot>) {
// //     while is_current_task(&bot, "`oBreak Cave Backgrounds`") {
// //         let dirt_tiles = {
// //             let world = &bot.world.read().unwrap();
// //             world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == DIRT).collect::<Vec<_>>()
// //         };
// //
// //         for tile in dirt_tiles.iter() {
// //             if !is_current_task(&bot, "`oBreak Cave Backgrounds`") {
// //                 return;
// //             }
// //
// //             while {
// //                 let world = &bot.world.read().unwrap();
// //                 let tile = world.get_tile(tile.x, tile.y).unwrap();
// //                 tile.background_item_id != 0 || tile.foreground_item_id != 0
// //             } {
// //                 core::find_path(&bot, tile.x, tile.y - 1);
// //                 thread::sleep(std::time::Duration::from_millis(100));
// //                 core::punch(&bot, 0, 1);
// //                 thread::sleep(std::time::Duration::from_millis(250));
// //             }
// //         }
// //     }
// // }
// //
// // pub fn collect_cave_background_seed(bot: &Arc<Bot>) {
// //     while is_current_task(&bot, "`oCollect Cave Background Seeds`") {
// //         let cbg_tree_tiles = {
// //             let world = &bot.world.read().unwrap();
// //             world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == CAVE_BACKGROUND_SEED).collect::<Vec<_>>()
// //         };
// //
// //         for tile in cbg_tree_tiles.iter() {
// //             if !is_current_task(&bot, "`oCollect Cave Background Seeds`") {
// //                 return;
// //             }
// //
// //             core::find_path(&bot, tile.x, tile.y);
// //             thread::sleep(std::time::Duration::from_millis(100));
// //             core::punch(&bot, 0, 0);
// //             thread::sleep(std::time::Duration::from_millis(250));
// //         }
// //
// //         core::find_path(&bot, 0, 0);
// //         thread::sleep(std::time::Duration::from_millis(100));
// //
// //         while is_current_task(&bot, "`oCollect Cave Background Seeds`") {
// //             core::place(&bot, 1, 0, ROCK as u32);
// //             thread::sleep(std::time::Duration::from_millis(100));
// //
// //             while {
// //                 let world = &bot.world.read().unwrap();
// //                 world.get_tile(1, 0).map_or(false, |tile| tile.foreground_item_id == CAVE_BACKGROUND)
// //             } {
// //                 core::punch(&bot, 1, 0);
// //                 thread::sleep(std::time::Duration::from_millis(250));
// //             }
// //         }
// //     }
// // }
// //
// // pub fn splice_rock_and_cbg_seed(bot: &Arc<Bot>) {
// //     if !is_current_task(&bot, "`oSplice Rock and Cave Background Seeds`") {
// //         return;
// //     }
// //
// //     core::find_path(&bot, 0, 0);
// //     thread::sleep(std::time::Duration::from_millis(100));
// //     core::place(&bot, 1, 1, ROCK as u32);
// //     thread::sleep(std::time::Duration::from_millis(100));
// //     core::place(&bot, 1, 0, CAVE_BACKGROUND_SEED as u32);
// //     thread::sleep(std::time::Duration::from_millis(100));
// //     core::place(&bot, 1, 0, ROCK_SEED as u32);
// //     thread::sleep(std::time::Duration::from_millis(100));
// // }
// //
// // pub fn place_sign_in_world(bot: &Arc<Bot>) {
// //     if !is_current_task(&bot, "`oPlace a Sign in the World`") {
// //         return;
// //     }
// //
// //     loop {
// //         let (sign_tree_tile, is_harvestable, sign_count) = {
// //             let world = &bot.world.read().unwrap();
// //             let inventory = &bot.inventory.read().unwrap();
// //             let sign_count = inventory.items.get(&SIGN).map_or(0, |item| item.amount);
// //             let tile = world.tiles.clone().into_iter().find(|tile| tile.foreground_item_id == SIGN_SEED).unwrap();
// //             let is_harvestable = world.is_tile_harvestable(&tile);
// //             (tile, is_harvestable, sign_count)
// //         };
// //
// //         if is_harvestable {
// //             core::find_path(&bot, sign_tree_tile.x, sign_tree_tile.y);
// //             thread::sleep(std::time::Duration::from_millis(100));
// //             core::punch(&bot, 0, 0);
// //             thread::sleep(std::time::Duration::from_millis(250));
// //             break;
// //         }
// //
// //         if sign_count > 0 {
// //             core::place(&bot, 0, 0, SIGN as u32);
// //             thread::sleep(std::time::Duration::from_millis(250));
// //         }
// //     }
// // }
// //
// // pub fn wrench_sign(bot: &Arc<Bot>) {
// //     if !is_current_task(&bot, "`oWrench the Sign that you placed`") {
// //         return;
// //     }
// //
// //     let sign_tile = {
// //         let world = &bot.world.read().unwrap();
// //         let tile = world.tiles.clone().into_iter().find(|tile| tile.foreground_item_id == SIGN);
// //         (tile)
// //     };
// //
// //     if sign_tile.is_some() {
// //         let sign_tile = sign_tile.unwrap();
// //         core::find_path(&bot, sign_tile.x, sign_tile.y);
// //         thread::sleep(std::time::Duration::from_millis(100));
// //         core::wrench(&bot, 0, 0);
// //         thread::sleep(std::time::Duration::from_millis(1000));
// //         core::send_packet(&bot, EPacketType::NetMessageGenericText, format!("action|dialog_return\ndialog_name|sign_edit\ntilex|{}|\ntiley|{}|\nsign_text|CLOEI\n", sign_tile.x, sign_tile.y).to_string());
// //         thread::sleep(std::time::Duration::from_millis(1000));
// //         while {
// //             let world = &bot.world.read().unwrap();
// //             world.get_tile(1, 0).map_or(false, |tile| tile.foreground_item_id == SIGN)
// //         } {
// //             core::punch(&bot, 0, 0);
// //             thread::sleep(std::time::Duration::from_millis(250));
// //         }
// //     }
// // }
// //
// // pub fn harvest_and_break_lava(bot: &Arc<Bot>) {
// //     if !is_current_task(&bot, "`oBreak Lava Blocks`") {
// //         return;
// //     }
// //
// //     let lava_tree_tiles = {
// //         let world = &bot.world.read().unwrap();
// //         let tiles = world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == LAVA_SEED).collect::<Vec<_>>();
// //         (tiles)
// //     };
// //
// //     for tile in lava_tree_tiles.iter() {
// //         let (is_harvestable, lava_count) = {
// //             let world = &bot.world.read().unwrap();
// //             let inventory = &bot.inventory.read().unwrap();
// //             let lava_count = inventory.items.get(&LAVA).map_or(0, |item| item.amount);
// //             (world.is_tile_harvestable(&tile), lava_count)
// //         };
// //
// //         if lava_count > 5 {
// //             break;
// //         }
// //
// //         if is_harvestable && lava_count < 5 {
// //             core::find_path(&bot, tile.x, tile.y);
// //             thread::sleep(std::time::Duration::from_millis(100));
// //             core::punch(&bot, 0, 0);
// //             thread::sleep(std::time::Duration::from_millis(250));
// //         }
// //     }
// //
// //     core::find_path(&bot, 0, 0);
// //     thread::sleep(std::time::Duration::from_millis(100));
// //     while is_current_task(&bot, "`oBreak Lava Blocks`") {
// //         core::place(&bot, 1, 0, LAVA as u32);
// //         thread::sleep(std::time::Duration::from_millis(250));
// //
// //         while {
// //             let world = &bot.world.read().unwrap();
// //             world.get_tile(1, 0).map_or(false, |tile| tile.foreground_item_id == LAVA)
// //         } {
// //             core::punch(&bot, 1, 0);
// //             thread::sleep(std::time::Duration::from_millis(250));
// //         }
// //     }
// // }
// //
// // pub fn collect_lava_seed(bot: &Arc<Bot>) {
// //     while is_current_task(bot, "`oCollect Lava Seeds`") {
// //         let lava_tree_tiles = {
// //             let world = bot.world.read().unwrap();
// //             world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == LAVA_SEED).collect::<Vec<_>>()
// //         };
// //
// //         for tile in lava_tree_tiles.iter() {
// //             if !is_current_task(bot, "`oCollect Lava Seeds`") {
// //                 return;
// //             }
// //
// //             core::find_path(bot, tile.x, tile.y);
// //             thread::sleep(std::time::Duration::from_millis(100));
// //             core::punch(bot, 0, 0);
// //             thread::sleep(std::time::Duration::from_millis(250));
// //         }
// //
// //         core::find_path(bot, 0, 0);
// //         thread::sleep(std::time::Duration::from_millis(100));
// //
// //         while is_current_task(bot, "`oCollect Lava Seeds`") {
// //             core::place(bot, 1, 0, LAVA as u32);
// //             thread::sleep(std::time::Duration::from_millis(100));
// //
// //             while {
// //                 let world = bot.world.read().unwrap();
// //                 world.get_tile(1, 0).map_or(false, |tile| tile.foreground_item_id == LAVA)
// //             } {
// //                 core::punch(bot, 1, 0);
// //                 thread::sleep(std::time::Duration::from_millis(250));
// //             }
// //         }
// //     }
// // }
// //
// // pub fn splice_lava_and_dirt_seed(bot: &Arc<Bot>) {
// //     if !is_current_task(&bot, "`oSplice Lava and Dirt Seeds`") {
// //         return;
// //     }
// //
// //     core::find_path(&bot, 2, 4);
// //     thread::sleep(std::time::Duration::from_millis(100));
// //     loop {
// //         let (lava_seed_count, dirt_seed_count) = {
// //             let inventory = bot.inventory.read().unwrap();
// //             let lava_seed_count = inventory.items.get(&LAVA_SEED).map_or(0, |item| item.amount);
// //             let dirt_seed_count = inventory.items.get(&DIRT_SEEDS).map_or(0, |item| item.amount);
// //             (lava_seed_count, dirt_seed_count)
// //         };
// //
// //         if lava_seed_count < 3 {
// //             break_item_from_inventory(&bot, LAVA, 3);
// //         }
// //
// //         if dirt_seed_count < 3 {
// //             break_item_from_inventory(&bot, DIRT, 3);
// //         }
// //
// //         if lava_seed_count >= 3 && dirt_seed_count >= 3 {
// //             break;
// //         }
// //     }
// //
// //     thread::sleep(std::time::Duration::from_millis(100));
// //     core::place(&bot, 0, -1, ROCK as u32);
// //     thread::sleep(std::time::Duration::from_millis(250));
// //     core::place(&bot, -1, -1, ROCK as u32);
// //     thread::sleep(std::time::Duration::from_millis(250));
// //     core::place(&bot, 1, -1, ROCK as u32);
// //     thread::sleep(std::time::Duration::from_millis(250));
// //
// //     core::place(&bot, 0, -2, LAVA_SEED as u32);
// //     thread::sleep(std::time::Duration::from_millis(250));
// //     core::place(&bot, -1, -2, LAVA_SEED as u32);
// //     thread::sleep(std::time::Duration::from_millis(250));
// //     core::place(&bot, 1, -2, LAVA_SEED as u32);
// //     thread::sleep(std::time::Duration::from_millis(250));
// //
// //     core::place(&bot, 0, -2, DIRT_SEEDS as u32);
// //     thread::sleep(std::time::Duration::from_millis(250));
// //     core::place(&bot, -1, -2, DIRT_SEEDS as u32);
// //     thread::sleep(std::time::Duration::from_millis(250));
// //     core::place(&bot, 1, -2, DIRT_SEEDS as u32);
// //     thread::sleep(std::time::Duration::from_millis(250));
// // }
// //
// // pub fn harvest_and_place_wood_block(bot: &Arc<Bot>) {
// //     if !is_current_task(&bot, "`oBuild Wood Blocks`") {
// //         return;
// //     }
// //
// //     let wood_tree_tiles = {
// //         let world = bot.world.read().unwrap();
// //         world.tiles.clone().into_iter().filter(|tile| tile.foreground_item_id == WOOD_BLOCK_SEED).collect::<Vec<_>>()
// //     };
// //
// //     for tile in wood_tree_tiles.iter() {
// //         let (is_harvestable, wood_count) = {
// //             let world = bot.world.read().unwrap();
// //             let inventory = bot.inventory.read().unwrap();
// //             let wood_count = inventory.items.get(&WOOD_BLOCK).map_or(0, |item| item.amount);
// //             (world.is_tile_harvestable(&tile), wood_count)
// //         };
// //
// //         if wood_count > 10 {
// //             break;
// //         }
// //
// //         if is_harvestable {
// //             core::find_path(&bot, tile.x, tile.y);
// //             thread::sleep(std::time::Duration::from_millis(100));
// //             core::punch(&bot, 0, 0);
// //             thread::sleep(std::time::Duration::from_millis(500));
// //         }
// //     }
// //
// //     core::find_path(&bot, 1, 7);
// //     for i in 0..10 {
// //         core::walk(&bot, i, 0, false);
// //         thread::sleep(std::time::Duration::from_millis(100));
// //         core::place(&bot, 0, -1, WOOD_BLOCK as u32);
// //         thread::sleep(std::time::Duration::from_millis(250));
// //     }
// // }
// //
// // pub fn break_item_from_inventory(bot: &Arc<Bot>, item_id: u16, target: u16) {
// //     while {
// //         let inventory = bot.inventory.read().unwrap();
// //         inventory.items.get(&(&item_id + 1)).map_or(0, |item| item.amount) < target
// //     } {
// //         core::place(&bot, 1, 0, item_id as u32);
// //         thread::sleep(std::time::Duration::from_millis(100));
// //         core::punch(bot, 1, 0);
// //         thread::sleep(std::time::Duration::from_millis(250));
// //     }
// // }
// //
// fn is_current_task(bot: &Arc<Bot>, task: &str) -> bool {
//     let ftue = bot.ftue.read().unwrap();
//     ftue.info.contains(task)
// }
// //
// // pub fn start(bot: &Arc<Bot>) {
// //     lock_the_world(&bot);
// //     break_dirt_block(&bot);
// //     plant_dirt_seed(&bot);
// //     harvest_dirt_tree(&bot);
// //     break_rock_block(&bot);
// //     collect_rock_seed(&bot);
// //     break_cave_background(&bot);
// //     collect_cave_background_seed(&bot);
// //     splice_rock_and_cbg_seed(&bot);
// //     place_sign_in_world(&bot);
// //     wrench_sign(&bot);
// //     harvest_and_break_lava(&bot);
// //     collect_lava_seed(&bot);
// //     splice_lava_and_dirt_seed(&bot);
// //     harvest_and_place_wood_block(&bot);
// // }
