// use std::sync::Arc;
// use std::thread;
// use std::time::Duration;
// use crate::core::{find_path, place, punch, Bot};
//
// pub fn place_dirt(bot: &Arc<Bot>) {
//     let tiles = {
//         let world = bot.world.read().unwrap();
//         world.tiles.clone()
//     };
//
//     for tile in tiles {
//         let local = {
//             let position = bot.position.read().unwrap();
//             position.clone()
//         };
//         if ((local.x / 32.0) as u32) == 97 && ((local.y / 32.0) as u32) == 97 {
//             if tile.y % 1 == 0 || tile.x == 0 || tile.x == 1 || tile.x == 98 || tile.x == 99 {
//                 if tile.foreground_item_id == 0 || (tile.background_item_id == 14 && tile.foreground_item_id == 0) {
//                     find_path(&bot, tile.x, tile.y - 1);
//                     thread::sleep(Duration::from_millis(100));
//                     place(&bot, 0, 0, 2);
//                     thread::sleep(Duration::from_millis(100));
//                 }
//             }
//         }
//     }
// }
//
// pub fn break_lava_block(bot: &Arc<Bot>) {
//     let tiles = {
//         let world = bot.world.read().unwrap();
//         world.tiles.clone()
//     };
//
//     for tile in tiles {
//         let local = {
//             let position = bot.position.read().unwrap();
//             position.clone()
//         };
//         if ((local.x / 32.0) as u32) == 97 && ((local.y / 32.0) as u32) == 51 {
//             if tile.y % 1 == 0 || tile.x == 0 || tile.x == 1 || tile.x == 98 || tile.x == 99 {
//                 if tile.foreground_item_id == 4 {
//                     find_path(&bot, tile.x, tile.y - 1);
//                     thread::sleep(Duration::from_millis(100));
//                     loop {
//                         let (foreground_id) = {
//                             let world = bot.world.read().unwrap();
//                             if let Some(tile) = world.get_tile(tile.x, tile.y) {
//                                 (tile.foreground_item_id)
//                             } else {
//                                 break;
//                             }
//                         };
//                         if foreground_id != 0 {
//                             punch(&bot, 0, 1);
//                             thread::sleep(Duration::from_millis(350));
//                         } else {
//                             break;
//                         }
//                     }
//                     // drop_seed(&core);
//                 }
//             }
//         }
//     }
// }
//
// pub fn break_dirt_block(bot: &Arc<Bot>) {
//     let tiles = {
//         let world = bot.world.read().unwrap();
//         world.tiles.clone()
//     };
//
//     for tile in tiles {
//         if tile.y % 2 == 1 || tile.x == 0 || tile.x == 1 || tile.x == 98 || tile.x == 99 {
//             if (tile.foreground_item_id == 2 || (tile.background_item_id == 14 && tile.foreground_item_id == 0) || tile.foreground_item_id == 10 || tile.foreground_item_id == 4) && tile.foreground_item_id != 8 {
//                 find_path(&bot, tile.x, tile.y - 2);
//                 thread::sleep(Duration::from_millis(100));
//                 loop {
//                     let (background_id) = {
//                         let world = bot.world.read().unwrap();
//                         if let Some(tile) = world.get_tile(tile.x, tile.y) {
//                             (tile.background_item_id)
//                         } else {
//                             break;
//                         }
//                     };
//                     if background_id != 0 {
//                         punch(&bot, 0, 2);
//                         thread::sleep(Duration::from_millis(350));
//                     } else {
//                         break;
//                     }
//                 }
//                 // drop_seed(&core);
//             }
//         }
//     }
// }
//
// pub fn start(bot: &Arc<Bot>) {
//     break_dirt_block(&bot);
//     break_lava_block(&bot);
//     place_dirt(&bot);
// }
