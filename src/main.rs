mod astar;
mod socks5;
mod lua_api;
mod constants;
mod cursor;
mod inventory;
mod bot;
pub mod events;
pub mod save_dat;
pub mod world;
mod bot_manager;
mod bot_state;
pub mod items;
mod player;
mod crypto;
mod dashboard;
mod login;
pub mod packet;
mod proxy_test;
mod server_data;
mod variant;
mod web;
pub mod script_channel;

use std::sync::{Arc, Mutex};
use bot_manager::BotManager;

#[tokio::main]
async fn main() {
    let (ws_tx, _) = tokio::sync::broadcast::channel(256);
    let mgr = Arc::new(Mutex::new(BotManager::new(ws_tx.clone())));
    web::serve(mgr, ws_tx).await;
}
