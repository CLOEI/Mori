use crate::core::Bot;
use crate::manager::proxy_manager::ProxyManager;
use crate::types::config::BotConfig;
use crate::utils;
use gtitem_r::structs::ItemDatabase;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::{spawn, JoinHandle};

pub struct BotManager {
    pub bots: Vec<(Arc<Bot>, JoinHandle<()>)>,
    pub items_database: Arc<RwLock<ItemDatabase>>,
    pub proxy_manager: Arc<RwLock<ProxyManager>>,
}

impl BotManager {
    pub fn new(proxy_manager: Arc<RwLock<ProxyManager>>) -> Self {
        let item_database = Arc::new(RwLock::new(ItemDatabase::new()));

        Self {
            bots: vec![],
            items_database: item_database,
            proxy_manager,
        }
    }
}

impl BotManager {
    pub fn add_bot(&mut self, bot: BotConfig) {
        let items_database_clone = Arc::clone(&self.items_database);
        let proxy_manager_clone = Arc::clone(&self.proxy_manager);

        let new_bot = Bot::new(bot.clone(), items_database_clone, proxy_manager_clone);
        // let newbot_clone = Arc::clone(&new_bot);

        let handle = spawn(move || {
            // newbot_clone.logon(bot.data.clone());
        });
        self.bots.push((new_bot, handle));
    }

    pub fn remove_bot(&mut self, username: &str) {
        let bot = self.get_bot(username);
        if let Some(bot) = bot {
            let bot_clone = Arc::clone(bot);
            thread::spawn(move || {
                let is_running = {
                    let state = bot_clone.state.lock().unwrap();
                    state.is_running
                };

                if is_running {
                    {
                        let mut state = bot_clone.state.lock().unwrap();
                        state.is_running = false;
                    }
                    bot_clone.disconnect();
                }
            });
            self.bots
                .retain(|(b, _)| b.info.lock().unwrap().payload[0] != username);
            utils::config::remove_bot(username.to_string());
        }
    }

    pub fn get_bot(&self, username: &str) -> Option<&Arc<Bot>> {
        for (bot, _) in &self.bots {
            if bot.info.lock().unwrap().payload[0] == username {
                return Some(bot);
            }
        }
        None
    }
}
