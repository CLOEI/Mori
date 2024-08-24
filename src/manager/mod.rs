use crate::bot::{self, Bot};
use crate::types::config::BotConfig;
use enet::Enet;
use gtitem_r::structs::ItemDatabase;
use paris::{error, info};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};

pub struct Manager {
    pub bots: Vec<(Arc<Bot>, JoinHandle<()>)>,
    pub items_database: Arc<ItemDatabase>,
    pub enet: Arc<Enet>,
}

impl Manager {
    pub fn new() -> Manager {
        let enet = Enet::new().expect("could not initialize ENet");
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
            enet: Arc::new(enet),
        }
    }
}

impl Manager {
    pub fn add_bot(&mut self, bot: BotConfig) {
        let items_database_clone = Arc::clone(&self.items_database);
        let enet_clone = Arc::clone(&self.enet);

        let new_bot = Arc::new(Bot::new(bot.clone(), enet_clone, items_database_clone));
        let newbot_clone = Arc::clone(&new_bot);

        let handle = spawn(move || {
            bot::logon(&newbot_clone, bot.data.clone());
        });
        self.bots.push((new_bot, handle));
    }

    pub fn remove_bot(&mut self, username: &str) {}

    pub fn get_bot(&self, username: &str) -> Option<&Arc<Bot>> {
        for (bot, _) in &self.bots {
            if bot.info.read().unwrap().username == username {
                return Some(bot);
            }
        }
        None
    }
}
