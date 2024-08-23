use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

use crate::bot::{self, Bot};
use crate::types::config::BotConfig;
use crate::types::elogin_method::ELoginMethod;
use gtitem_r::structs::ItemDatabase;
use paris::{error, info};

pub struct Manager {
    pub bots: Vec<(Arc<Bot>, JoinHandle<()>)>,
    pub items_database: Arc<ItemDatabase>,
}

impl Manager {
    pub fn new() -> Manager {
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

        Manager {
            bots: vec![],
            items_database: Arc::new(item_database),
        }
    }
}

impl Manager {
    pub fn add_bot(&mut self, bot: BotConfig) {
        let items_database_clone = Arc::clone(&self.items_database);

        let new_bot = Arc::new(Bot::new(bot.clone(), items_database_clone));
        let newbot_clone = Arc::clone(&new_bot);

        let handle = spawn(move || {
            bot::logon(&newbot_clone, bot.data.clone());
        });
        self.bots.push((new_bot, handle));
    }

    pub fn remove_bot(&mut self, username: &str) {}

    pub fn get_bot(&self, username: &str) -> Option<&Arc<Bot>> {
        for (bot, _) in &self.bots {
            if bot.info.lock().unwrap().username == username {
                return Some(bot);
            }
        }
        None
    }
}
