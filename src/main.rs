use std::sync::Arc;
use std::sync::RwLock;
use gt_core::{Bot, gtitem_r};

fn main() {
    let item_database = Arc::new(RwLock::new(gtitem_r::load_from_file("items.dat").unwrap()));
    println!("Loaded {} items from items.dat", item_database.read().unwrap().item_count);
    let (bot, _) = Bot::new(gt_core::types::bot::LoginVia::LEGACY(["".into(), "".into()]), None, item_database.clone(), None);
    bot.logon(None);

    loop {}
}