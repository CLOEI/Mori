mod astar;
mod auth;
mod bot;
pub mod constants;
mod cursor;
mod inventory;
pub mod events;
mod bot_state;
mod bot_manager;
mod dashboard;
pub mod items;
mod login;
mod lua;
mod player;
mod protocol;
mod proxy_test;
pub mod save_dat;
mod server_data;
pub mod setup;
mod socks5;
pub mod world;
mod web;
pub mod script_channel;

use std::sync::{Arc, Mutex};
use bot_manager::BotManager;

#[tokio::main]
async fn main() {
    setup::init();
    let (ws_tx, _) = tokio::sync::broadcast::channel(256);
    let mgr = Arc::new(Mutex::new(BotManager::new(ws_tx.clone())));
    web::serve(mgr, ws_tx).await;
}
