use std::sync::{Arc, Mutex, RwLock};
use std::thread::{spawn, JoinHandle};

use crate::bot::{self, Bot};
use crate::types::e_login_method::ELoginMethod;
use gtitem_r::structs::ItemDatabase;
use spdlog::prelude::*;

pub struct Manager {
    pub bots: Vec<(Arc<Mutex<Bot>>, JoinHandle<()>)>,
    pub items_database: Arc<ItemDatabase>,
}

impl Manager {
    pub fn new() -> Result<Manager, String> {
        info!("Loading items database...");
        let item_database = gtitem_r::load_from_file("items.dat").unwrap();
        info!("Successfully loaded items database");
        info!("Initialized Manager");

        Ok(Manager {
            bots: vec![],
            items_database: Arc::new(item_database),
        })
    }
}

impl Manager {
    pub fn add_bot(
        &mut self,
        username: String,
        password: String,
        code: String,
        method: ELoginMethod,
    ) {
        if method == ELoginMethod::LEGACY {
            info!("Adding bot: {}", username);
        } else {
            info!("Adding bot with method: {:?}", method);
        }
        let items_database_clone = Arc::clone(&self.items_database);
        let new_bot = Arc::new(Mutex::new(Bot::new(
            username,
            password,
            code,
            method,
            items_database_clone,
        )));
        let newbot_clone = Arc::clone(&new_bot);

        let handle = spawn(move || {
            bot::login(newbot_clone);
        });
        self.bots.push((new_bot, handle));
    }
    pub fn remove_bot(&mut self, username: &str) {}
    pub fn get_bot(&self, username: &str) -> Option<&Arc<Mutex<Bot>>> {
        for (bot, _) in &self.bots {
            let bot_mutex = bot.lock().unwrap();
            if bot_mutex.info.username == username {
                return Some(bot);
            }
        }
        None
    }
}
