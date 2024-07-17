use std::sync::Arc;

use crate::bot::{get_oauth_links, Bot};
use crate::types::e_login_method::ELoginMethod;
use gtitem_r::structs::ItemDatabase;
use spdlog::prelude::*;

pub struct Manager {
    bots: Vec<Box<Bot>>,
    oauth_links: Vec<String>,
    items_database: Arc<ItemDatabase>,
}

impl Manager {
    pub fn new() -> Result<Manager, String> {
        info!("Getting OAuth links...");
        let links = match get_oauth_links() {
            Ok(links) => links,
            Err(err) => return Err(err.to_string()),
        };
        if links.len() < 3 {
            return Err("Something's wrong".to_string());
        }
        info!("Successfully got OAuth links for: apple, google and legacy");

        info!("Loading items database...");
        let item_database = gtitem_r::load_from_file("items.dat").unwrap();
        info!("Successfully loaded items database");
        info!("Initialized Manager");

        Ok(Manager {
            bots: Vec::new(),
            oauth_links: links,
            items_database: Arc::new(item_database),
        })
    }
}

impl Manager {
    pub fn add_bot(&mut self, username: &str, password: &str, method: ELoginMethod) {
        info!("Adding bot: {}", username);
        let mut bot = Bot::new(
            username.to_string(),
            password.to_string(),
            method,
            self.oauth_links.clone(),
            Arc::clone(&self.items_database),
        );
        bot.login();
        self.bots.push(Box::new(bot));
    }
    pub fn remove_bot(&mut self, username: &str) {}
    pub fn get_bot(&self, username: &str) {}
}
