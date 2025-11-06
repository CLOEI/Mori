use std::sync::{Arc, RwLock};
use std::thread::{JoinHandle, spawn};

use gt_core::gtitem_r::load_from_file;
use gt_core::gtitem_r::structs::ItemDatabase;
use gt_core::types::bot::LoginVia;
use gt_core::{Bot, Socks5Config};

pub struct BotManager {
    pub bots: Vec<(Arc<Bot>, JoinHandle<()>)>,
    pub items_database: Arc<RwLock<ItemDatabase>>,
}

pub struct BotConfig {
    pub login_method: LoginVia,
    pub proxy: Option<Socks5Config>,
}

impl BotManager {
    pub fn new() -> Self {
        let item_database = load_from_file("items.dat").expect("items.dat missing/failed to parse");
        let item_database = Arc::new(RwLock::new(item_database));

        Self {
            bots: vec![],
            items_database: item_database,
        }
    }
}

impl BotManager {
    pub fn add_bot(manager: Arc<RwLock<Self>>, bot: BotConfig) {
        std::thread::spawn(move || {
            let items_database = {
                let manager_guard = manager.read().unwrap();
                Arc::clone(&manager_guard.items_database)
            };

            let BotConfig {
                login_method,
                proxy,
            } = bot;

            let (bot_instance, _) = Bot::new(login_method, None, items_database, proxy);
            let bot_clone = bot_instance.clone();

            let handle = spawn(move || {
                bot_clone.logon(None);
            });

            let mut manager_guard = manager.write().unwrap();
            manager_guard.bots.push((bot_instance, handle));
        });
    }

    pub fn remove_bot(&mut self, username: &str) {
        todo!("Implement bot removal");
    }

    pub fn get_bot(&self, username: &str) -> Option<&Arc<Bot>> {
        for (bot, _) in &self.bots {
            if let Some(login_info_guard) = bot.auth.try_login_info() {
                if let Some(login_info) = login_info_guard.as_ref() {
                    if login_info.tank_id_name == username {
                        return Some(bot);
                    }
                }
            }
        }
        None
    }
}
