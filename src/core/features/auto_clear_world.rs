// use crate::core::Bot;
// use std::sync::Arc;
// use std::thread;
// use std::time::Duration;

// static CAVE_BACKGROUND: u16 = 14;
// static BEDROCK: u16 = 8;

// pub fn start(bot: &Arc<Bot>) {
//     let (world_width, world_height) = {
//         let world = bot.world.read().unwrap();
//         (world.width, world.height)
//     };

//     {
//         let position = {
//             let position = bot.position.read().unwrap();
//             position.clone()
//         };
//         while bot.is_inworld() {
//             let (foreground_id, background_id) = {
//                 let world = bot.world.read().unwrap();
//                 let tile = world
//                     .get_tile((position.x / 32.0) as u32, (position.y / 32.0 + 2.0) as u32)
//                     .unwrap();
//                 (tile.foreground_item_id, tile.background_item_id)
//             };

//             if background_id != CAVE_BACKGROUND || foreground_id == BEDROCK {
//                 break;
//             }
//             bot.punch(0, 2);
//             thread::sleep(Duration::from_millis(350));
//         }
//     }

//     for y in 23..world_height - 6 {
//         for x in 0..world_width {
//             while bot.is_inworld() {
//                 let (foreground_id, background_id) = {
//                     let world = bot.world.read().unwrap();
//                     if let Some(tile) = world.get_tile(x, y) {
//                         (tile.foreground_item_id, tile.background_item_id)
//                     } else {
//                         break;
//                     }
//                 };

//                 if background_id != CAVE_BACKGROUND || foreground_id == BEDROCK {
//                     break;
//                 }
//                 bot.find_path(x, y - 1);
//                 thread::sleep(Duration::from_millis(100));
//                 bot.punch(0, 1);
//                 thread::sleep(Duration::from_millis(350));
//             }
//         }
//     }
// }
