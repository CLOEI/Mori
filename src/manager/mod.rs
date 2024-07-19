use std::sync::Arc;

use crate::bot::Bot;
use crate::types::e_login_method::ELoginMethod;
use gtitem_r::structs::ItemDatabase;
use spdlog::prelude::*;

pub struct Manager {
    bots: Vec<Box<Bot>>,
    items_database: Arc<ItemDatabase>,
}

impl Manager {
    pub fn new() -> Result<Manager, String> {
        info!("Loading items database...");
        let item_database = gtitem_r::load_from_file("items.dat").unwrap();
        info!("Successfully loaded items database");
        info!("Initialized Manager");

        Ok(Manager {
            bots: Vec::new(),
            items_database: Arc::new(item_database),
        })
    }
}

impl Manager {
    pub fn add_bot(&mut self, username: &str, password: &str, method: ELoginMethod) {
        if method == ELoginMethod::LEGACY {
            info!("Adding bot: {}", username);
        } else {
            info!("Adding bot with method: {:?}", method);
        }
        let mut bot = Bot::new(
            username.to_string(),
            password.to_string(),
            method,
            Arc::clone(&self.items_database),
        );
        bot.login();
        self.bots.push(Box::new(bot));
    }
    pub fn remove_bot(&mut self, username: &str) {}
    pub fn get_bot(&self, username: &str) {}
}
