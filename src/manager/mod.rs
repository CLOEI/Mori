use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

use crate::bot::Bot;
use crate::types::e_login_method::ELoginMethod;
use gtitem_r::structs::ItemDatabase;
use spdlog::prelude::*;

pub struct Manager {
    pub bots: Mutex<Vec<JoinHandle<()>>>,
    items_database: Arc<ItemDatabase>,
}

impl Manager {
    pub fn new() -> Result<Manager, String> {
        info!("Loading items database...");
        let item_database = gtitem_r::load_from_file("items.dat").unwrap();
        info!("Successfully loaded items database");
        info!("Initialized Manager");

        Ok(Manager {
            bots: Mutex::new(Vec::new()),
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
        let handle = spawn(move || {
            let mut bot = Bot::new(username, password, code, method, items_database_clone);
            bot.login();
        });
        self.bots.lock().unwrap().push(handle);
    }
    pub fn remove_bot(&mut self, username: &str) {}
    pub fn get_bot(&self, username: &str) {}
}
