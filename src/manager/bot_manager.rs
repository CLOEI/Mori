use crate::bot::{self, Bot};
use crate::types::config::BotConfig;
use gtitem_r::structs::ItemDatabase;
use paris::{error, info};
use std::sync::Arc;
use std::thread;
use std::thread::{spawn, JoinHandle};
use crate::utils;

pub struct BotManager {
    pub bots: Vec<(Arc<Bot>, JoinHandle<()>)>,
    pub items_database: Arc<ItemDatabase>,
}

impl BotManager {
    pub fn new() -> Self {
        let item_database = {
            match gtitem_r::load_from_file("items.dat") {
                Ok(item_database) => {
                    info!("Item database loaded successfully");
                    item_database
                }
                Err(e) => {
                    error!("Failed to load item database: {}", e);
                    panic!("Failed to load item database: {}", e);
                }
            }
        };

        Self {
            bots: vec![],
            items_database: Arc::new(item_database),
        }
    }
}

impl BotManager {
    pub fn add_bot(&mut self, bot: BotConfig) {
        let items_database_clone = Arc::clone(&self.items_database);

        let new_bot = Arc::new(Bot::new(bot.clone(), items_database_clone));
        let newbot_clone = Arc::clone(&new_bot);

        let handle = spawn(move || {
            bot::logon(&newbot_clone, bot.data.clone());
        });
        self.bots.push((new_bot, handle));
    }

    pub fn remove_bot(&mut self, username: &str) {
        let bot = self.get_bot(username);
        if let Some(bot) = bot {
            let bot_clone = Arc::clone(bot);
            thread::spawn(move || {
                let is_running = {
                    let state = bot_clone.state.read().unwrap();
                    state.is_running
                };

                if is_running {
                    bot_clone.state.write().unwrap().is_running = false;
                    bot::disconnect(&bot_clone);
                }
            });
            self.bots.retain(|(b, _)| b.info.read().unwrap().payload[0] != username);
            utils::config::remove_bot(username.to_string());
        }
    }

    pub fn get_bot(&self, username: &str) -> Option<&Arc<Bot>> {
        for (bot, _) in &self.bots {
            if bot.info.read().unwrap().payload[0] == username {
                return Some(bot);
            }
        }
        None
    }
}
