use crate::connect::{get_oauth_links, Connect};
use crate::types::e_login_method::ELoginMethod;
use spdlog::prelude::*;

pub struct Manager {
    bots: Vec<Box<Connect>>,
    oauth_links: Vec<String>,
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
        info!("Initialized Manager");

        Ok(Manager {
            bots: Vec::new(),
            oauth_links: links,
        })
    }
}

impl Manager {
    pub fn add_bot(&mut self, username: &str, password: &str, method: ELoginMethod) {
        info!("Adding bot: {}", username);
        let mut bot = Connect::new(
            username.to_string(),
            password.to_string(),
            method,
            self.oauth_links.clone(),
        );
        bot.start();
        self.bots.push(Box::new(bot));
    }
    pub fn remove_bot(&mut self, username: &str) {}
    pub fn get_bot(&self, username: &str) {}
}
