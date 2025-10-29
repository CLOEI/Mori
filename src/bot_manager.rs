use gt_core::{Bot, Socks5Config};
use gt_core::gtitem_r;
use gt_core::gtitem_r::structs::ItemDatabase;
use gt_core::types::bot::LoginVia;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;
use uuid::Uuid;

pub struct BotEntry {
    pub id: Uuid,
    pub bot: Arc<Bot>,
    pub login_method: String,
}

pub struct BotManager {
    bots: RwLock<HashMap<Uuid, BotEntry>>,
    item_database: Arc<RwLock<ItemDatabase>>,
}

impl BotManager {
    pub fn new() -> Self {
        // Load item database
        let item_database = if Path::new("items.dat").exists() {
            gtitem_r::load_from_file("items.dat").unwrap_or_else(|_| ItemDatabase::new())
        } else {
            ItemDatabase::new()
        };

        Self {
            bots: RwLock::new(HashMap::new()),
            item_database: Arc::new(RwLock::new(item_database)),
        }
    }

    pub fn create_bot(
        &self,
        login_via: LoginVia,
        token_fetcher: Option<Box<dyn Fn(String, String) -> String + Send + Sync>>,
        socks5_config: Option<Socks5Config>,
    ) -> Result<Uuid, String> {
        let bot_id = Uuid::new_v4();
        let item_db = Arc::clone(&self.item_database);

        let bot = Bot::new(login_via.clone(), token_fetcher, item_db, socks5_config);

        let login_method = match login_via {
            LoginVia::GOOGLE => "Google".to_string(),
            LoginVia::APPLE => "Apple".to_string(),
            LoginVia::LTOKEN(_) => "LTOKEN".to_string(),
            LoginVia::LEGACY(_) => "Legacy".to_string(),
        };

        let bot_entry = BotEntry {
            id: bot_id,
            bot: Arc::clone(&bot),
            login_method,
        };

        // Spawn bot thread
        let bot_clone = Arc::clone(&bot);
        thread::spawn(move || {
            bot_clone.logon(None);
        });

        // Add to manager
        let mut bots = self.bots.write().unwrap();
        bots.insert(bot_id, bot_entry);

        Ok(bot_id)
    }

    pub fn get_bot(&self, id: &Uuid) -> Option<Arc<Bot>> {
        let bots = self.bots.read().unwrap();
        bots.get(id).map(|entry| Arc::clone(&entry.bot))
    }

    pub fn get_bot_entry(&self, id: &Uuid) -> Option<BotEntry> {
        let bots = self.bots.read().unwrap();
        bots.get(id).map(|entry| BotEntry {
            id: entry.id,
            bot: Arc::clone(&entry.bot),
            login_method: entry.login_method.clone(),
        })
    }

    pub fn list_bots(&self) -> Vec<(Uuid, String, Arc<Bot>)> {
        let bots = self.bots.read().unwrap();
        bots.iter()
            .map(|(id, entry)| (*id, entry.login_method.clone(), Arc::clone(&entry.bot)))
            .collect()
    }

    pub fn remove_bot(&self, id: &Uuid) -> Result<(), String> {
        let mut bots = self.bots.write().unwrap();

        if bots.remove(id).is_some() {
            // TODO: Disconnect bot
            Ok(())
        } else {
            Err("Bot not found".to_string())
        }
    }

    pub fn get_item_database(&self) -> Arc<RwLock<ItemDatabase>> {
        Arc::clone(&self.item_database)
    }
}
