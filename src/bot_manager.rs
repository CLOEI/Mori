use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;

use crate::bot::{Socks5Config};
use crate::bot_state::{BotCommand, BotState, BotStatus, CmdSender};
use crate::events::{WsEvent, WsTx};
use crate::items::ItemsDat;

pub struct BotEntry {
    pub username:         String,
    pub stop_flag:        Arc<AtomicBool>,
    pub state:            Arc<RwLock<BotState>>,
    pub cmd_tx:           CmdSender,
}

pub struct BotManager {
    next_id:   u32,
    pub bots:  HashMap<u32, BotEntry>,
    pub items_dat: Arc<ItemsDat>,
    pub ws_tx: WsTx,
}

#[derive(serde::Serialize)]
pub struct BotInfo {
    pub id:       u32,
    pub username: String,
    pub status:  String,
    pub world:   String,
    pub pos_x:   f32,
    pub pos_y:   f32,
    pub gems:    i32,
    pub ping_ms: u32,
}

impl BotManager {
    pub fn new(ws_tx: WsTx) -> Self {
        Self { next_id: 0, bots: HashMap::new(), items_dat: Arc::new(ItemsDat::load()), ws_tx }
    }

    pub fn spawn(&mut self, username: String, password: String, proxy: Option<Socks5Config>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let stop_flag   = Arc::new(AtomicBool::new(false));
        let stop_clone  = stop_flag.clone();
        let uname       = username.clone();
        let pass        = password.clone();

        let state = Arc::new(RwLock::new(BotState {
            status: BotStatus::Connecting,
            ..Default::default()
        }));
        let state_clone = state.clone();

        let (cmd_tx, cmd_rx) = mpsc::channel::<BotCommand>();

        let items_dat = self.items_dat.clone();
        let ws_tx_clone = self.ws_tx.clone();

        std::thread::spawn(move || {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut bot = crate::bot::Bot::new(&uname, &pass, proxy, state_clone, cmd_rx, items_dat, id, Some(ws_tx_clone));
                bot.run(stop_clone);
            })) {
                Ok(_)  => println!("[Bot:{id}] Stopped."),
                Err(_) => println!("[Bot:{id}] Crashed."),
            }
        });

        self.bots.insert(id, BotEntry { username: username.clone(), stop_flag, state, cmd_tx });
        let _ = self.ws_tx.send(WsEvent::BotAdded { bot_id: id, username });
        id
    }

    pub fn spawn_ltoken(&mut self, ltoken_str: String, proxy: Option<Socks5Config>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let stop_flag  = Arc::new(AtomicBool::new(false));
        let stop_clone = stop_flag.clone();

        let state = Arc::new(RwLock::new(BotState {
            status: BotStatus::Connecting,
            ..Default::default()
        }));
        let state_clone = state.clone();

        let (cmd_tx, cmd_rx) = mpsc::channel::<BotCommand>();

        let items_dat = self.items_dat.clone();
        let ws_tx_clone = self.ws_tx.clone();

        std::thread::spawn(move || {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut bot = crate::bot::Bot::new_ltoken(&ltoken_str, proxy, state_clone, cmd_rx, items_dat, id, Some(ws_tx_clone));
                bot.run(stop_clone);
            })) {
                Ok(_)  => println!("[Bot:{id}] Stopped."),
                Err(_) => println!("[Bot:{id}] Crashed."),
            }
        });

        self.bots.insert(id, BotEntry { username: String::new(), stop_flag, state, cmd_tx });
        let _ = self.ws_tx.send(WsEvent::BotAdded { bot_id: id, username: String::new() });
        id
    }

    pub fn stop(&mut self, id: u32) -> bool {
        if let Some(entry) = self.bots.remove(&id) {
            entry.stop_flag.store(true, Ordering::Relaxed);
            let _ = self.ws_tx.send(WsEvent::BotRemoved { bot_id: id });
            true
        } else {
            false
        }
    }

    pub fn list(&self) -> Vec<BotInfo> {
        self.bots.iter().map(|(id, e)| {
            let s = e.state.read().unwrap();
            BotInfo {
                id:       *id,
                username: e.username.clone(),
                status:  s.status.to_string(),
                world:   s.world_name.clone(),
                pos_x:   s.pos_x,
                pos_y:   s.pos_y,
                gems:    s.gems,
                ping_ms: s.ping_ms,
            }
        }).collect()
    }

    pub fn get_state(&self, id: u32) -> Option<BotState> {
        self.bots.get(&id).map(|e| e.state.read().unwrap().clone())
    }

    pub fn send_cmd(&self, id: u32, cmd: BotCommand) -> bool {
        self.bots.get(&id).map(|e| e.cmd_tx.send(cmd).is_ok()).unwrap_or(false)
    }

    pub fn run_script(&self, id: u32, content: String) -> bool {
        self.send_cmd(id, BotCommand::RunScript { content })
    }

    pub fn find_by_name(&self, name: &str) -> Option<(Arc<RwLock<BotState>>, CmdSender)> {
        self.bots.values()
            .find(|e| e.username.eq_ignore_ascii_case(name))
            .map(|e| (e.state.clone(), e.cmd_tx.clone()))
    }

    pub fn stop_by_name(&mut self, name: &str) -> bool {
        if let Some(id) = self.bots.iter()
            .find(|(_, e)| e.username.eq_ignore_ascii_case(name))
            .map(|(id, _)| *id)
        {
            self.stop(id)
        } else {
            false
        }
    }
}
